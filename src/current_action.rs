use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    component::{Component, ComponentId},
    error::Result,
    lifecycle::HookContext,
    prelude::ReflectComponent,
    system::EntityCommands,
    world::{DeferredWorld, EntityWorldMut},
};
use bevy_reflect::Reflect;
use thiserror::Error;

#[cfg(feature = "target")]
use crate::GotoTarget;
use crate::{ActionProviderTrait, SensorValue, sensor_state::SensorId};

#[derive(Error, Debug)]
pub enum InsertCurrentActionError {
    #[error("could not get sensor state")]
    SensorStateNotFound,
    #[error("could not get current target for sensor {0:?}")]
    CurrentTargetNotFound(SensorId),
    #[error("current target not a valid TargetSensor: {0:?}")]
    InvalidTargetValue(SensorValue),
}

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

pub trait ActionCommands {
    fn insert_action(&mut self, action: &dyn ActionProviderTrait);
    fn insert_current_action(&mut self, action: Box<dyn ActionProviderTrait>);
    fn remove_current_action(&mut self);
}

impl ActionCommands for EntityCommands<'_> {
    fn insert_action(&mut self, action: &dyn ActionProviderTrait) {
        let cloned_action = action.clone_box();

        self.queue(move |mut entity_world: EntityWorldMut| {
            cloned_action.insert_current_action(&mut entity_world);
        });
    }

    fn insert_current_action(&mut self, action: Box<dyn ActionProviderTrait>) {
        #[cfg(feature = "target")]
        if let Some(target) = *action.target() {
            self.queue(move |mut entity_world: EntityWorldMut| -> Result {
                use crate::{SensorState, SensorValue, world_sensor::TargetValue};

                let sensor_state = entity_world
                    .get::<SensorState>()
                    .ok_or(InsertCurrentActionError::SensorStateNotFound)?;
                let current_target = sensor_state
                    .get(&target.id)
                    .ok_or(InsertCurrentActionError::CurrentTargetNotFound(target.id))?;
                let entity = match current_target {
                    SensorValue::Target(Some(TargetValue { entity, .. })) => Ok(entity),
                    _ => Err(InsertCurrentActionError::InvalidTargetValue(
                        *current_target,
                    )),
                }?;

                if current_target.is_close(target.value) {
                    action.insert_current_action(&mut entity_world);
                } else {
                    entity_world.insert(CurrentAction {
                        action: GotoTarget {
                            next_action: Some(action),
                            target: *entity,
                        },
                    });
                }
                Ok(())
            });
        } else {
            self.insert_action(&*action);
        }
        #[cfg(not(feature = "target"))]
        self.insert_action(&action);
    }

    fn remove_current_action(&mut self) {
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
