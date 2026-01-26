use bevy_ecs::{
    component::Component,
    entity::Entity,
    error::Result,
    query::{Changed, Or},
    relationship::Relationship,
    system::{Commands, Query},
};
use thiserror::Error;

use crate::{
    ActionProviderTrait, CurrentAction, SensorState, SensorValue, TargetValue,
    current_action::CurrentActionCommands,
};

#[derive(Clone, PartialEq)]
pub enum TargetConfig {
    Proximity,
    Target,
}

pub struct GotoTarget {
    pub(crate) next_action: Option<Box<dyn ActionProviderTrait>>,
    pub(crate) target: Entity,
}

impl CurrentAction<GotoTarget> {
    pub fn target(&self) -> Entity {
        self.action.target
    }
    pub fn next_action(&self) -> Option<&Box<dyn ActionProviderTrait>> {
        self.action.next_action.as_ref()
    }
}

impl Relationship for CurrentAction<GotoTarget> {
    type RelationshipTarget = TargetedBy;

    fn get(&self) -> Entity {
        self.action.target
    }

    fn from(entity: Entity) -> Self {
        Self {
            action: GotoTarget {
                next_action: None,
                target: entity,
            },
        }
    }

    fn set_risky(&mut self, entity: Entity) {
        self.action.target = entity
    }
}

#[derive(Component)]
#[relationship_target(relationship = CurrentAction<GotoTarget>, linked_spawn)]
pub struct TargetedBy(Vec<Entity>);

#[derive(Error, Debug)]
pub enum GotoError {
    #[error("Goto target not found")]
    TargetNotFound,
    #[error("Goto action not found")]
    ActionNotFound,
}

pub fn finish_goto(
    mut commands: Commands,
    query: Query<
        (Entity, &CurrentAction<GotoTarget>, &SensorState),
        Or<(Changed<CurrentAction<GotoTarget>>, Changed<SensorState>)>,
    >,
) -> Result {
    for (entity, goto_action, sensor_state) in &query {
        let sensor_state = sensor_state.get(
            &goto_action
                .next_action()
                .ok_or(GotoError::ActionNotFound)?
                .target()
                .as_ref()
                .ok_or(GotoError::TargetNotFound)?
                .id,
        );
        if let SensorValue::Target(Some(TargetValue { is_close, .. })) = sensor_state.unwrap() {
            if *is_close {
                commands
                    .entity(entity)
                    .insert_action(goto_action.next_action().ok_or(GotoError::ActionNotFound)?);
            }
        }
    }
    Ok(())
}
