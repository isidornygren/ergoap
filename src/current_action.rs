use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    prelude::ReflectComponent,
    system::EntityCommands,
    world::{DeferredWorld, EntityWorldMut},
};
use bevy_reflect::Reflect;

use crate::ActionProviderTrait;
#[cfg(feature = "target")]
use crate::GotoTarget;

pub fn on_insert_current_action(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(prev_action_ref) = world
        .entity(entity)
        .get::<CurrentActionRef>()
        .map(|action_ref| action_ref.0)
    {
        if prev_action_ref != component_id {
            world
                .commands()
                .entity(entity)
                .remove_by_id(prev_action_ref)
                .insert(CurrentActionRef(component_id));
        }
    } else {
        world
            .commands()
            .entity(entity)
            .insert(CurrentActionRef(component_id));
    }
}

#[derive(Component, Debug)]
pub struct CurrentActionRef(ComponentId);

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[component(on_insert=on_insert_current_action)]
pub struct CurrentAction<A> {
    pub(crate) action: A,
}

impl<A> Deref for CurrentAction<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.action
    }
}

impl<A> DerefMut for CurrentAction<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.action
    }
}

pub trait CurrentActionCommands {
    fn spawn_current_action(&mut self, action: Box<dyn ActionProviderTrait>);
    fn despawn_current_action(&mut self);
    fn insert_action(&mut self, action: &dyn ActionProviderTrait);
}

impl CurrentActionCommands for EntityCommands<'_> {
    fn insert_action(&mut self, action: &dyn ActionProviderTrait) {
        let cloned_action = action.clone_box();
        self.queue(move |mut entity_world: EntityWorldMut| {
            cloned_action.insert_current_action(&mut entity_world);
        });
    }

    fn spawn_current_action(&mut self, action: Box<dyn ActionProviderTrait>) {
        #[cfg(feature = "target")]
        if let Some(target) = *action.target() {
            self.queue(move |mut entity_world: EntityWorldMut| {
                use crate::{SensorState, SensorValue, world_sensor::TargetValue};

                let sensor_state = entity_world
                    .get::<SensorState>()
                    .expect("Could not get sensor state");
                let current_target = sensor_state
                    .get(&target)
                    .expect("Could not get current target");
                let (entity, is_close) = match current_target {
                    SensorValue::Target(Some(TargetValue { entity, is_close })) => {
                        Some((entity, is_close))
                    }
                    _ => None,
                }
                .expect("Current target not the correct value");
                if *is_close {
                    action.insert_current_action(&mut entity_world);
                } else {
                    entity_world.insert(CurrentAction {
                        action: GotoTarget {
                            next_action: Some(action),
                            target: *entity,
                        },
                    });
                }
            });
        } else {
            self.insert_action(&*action);
        }
        #[cfg(not(feature = "target"))]
        self.insert_action(&action);
    }

    fn despawn_current_action(&mut self) {
        self.queue(|mut entity_world: EntityWorldMut| {
            if let Some(component_id) = entity_world
                .get::<CurrentActionRef>()
                .map(|action_ref| action_ref.0)
            {
                entity_world.remove_by_id(component_id);
                entity_world.remove::<CurrentActionRef>();
            }
        });
    }
}
