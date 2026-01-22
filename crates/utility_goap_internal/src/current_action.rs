use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    prelude::ReflectComponent,
    reflect::ReflectCommandExt,
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
    #[cfg(feature = "target")]
    fn force_spawn_current_action(&mut self, action: &Box<dyn ActionProviderTrait>);
    fn spawn_current_action(&mut self, action: Box<dyn ActionProviderTrait>);
    fn despawn_current_action(&mut self);
}

impl CurrentActionCommands for EntityCommands<'_> {
    #[cfg(feature = "target")]
    fn force_spawn_current_action(&mut self, action: &Box<dyn ActionProviderTrait>) {
        self.insert_reflect(action.component().to_dynamic());
    }
    fn spawn_current_action(&mut self, action: Box<dyn ActionProviderTrait>) {
        #[cfg(feature = "target")]
        if let Some(target_component_id) = action.target().map(|id| id.clone()) {
            self.queue(move |mut entity_world: EntityWorldMut| {
                use crate::{SensorState, SensorValue, world_sensor::TargetValue};

                let sensor_state = entity_world
                    .get::<SensorState>()
                    .expect("Could not get sensor state");
                let current_target = sensor_state
                    .get(&target_component_id)
                    .expect("Could not get current target");
                let (entity, is_close) = match current_target {
                    SensorValue::Target(Some(TargetValue { entity, is_close })) => {
                        Some((entity, is_close))
                    }
                    _ => None,
                }
                .expect("Current target not the correct value");
                if !is_close {
                    entity_world.insert(CurrentAction {
                        action: GotoTarget {
                            action: Some(action),
                            target: *entity,
                        },
                    });
                } else {
                    entity_world.insert_reflect(action.component().to_dynamic());
                }
            });
        } else {
            self.insert_reflect(action.component().to_dynamic());
        }
        #[cfg(not(feature = "target"))]
        self.insert_reflect(action.component().to_dynamic());
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
