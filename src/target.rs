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
    ActionProviderTrait, SensorState, current_action::ActionCommands, sensor_state::SensorId,
};

#[derive(Component)]
pub struct GotoTarget {
    pub(crate) next_action: Option<Box<dyn ActionProviderTrait>>,
    pub(crate) target: Entity,
}

impl GotoTarget {
    #[must_use]
    pub const fn target(&self) -> Entity {
        self.target
    }
    #[must_use]
    pub fn next_action(&self) -> Option<&dyn ActionProviderTrait> {
        self.next_action.as_deref()
    }
}

impl Relationship for GotoTarget {
    type RelationshipTarget = TargetedBy;

    fn get(&self) -> Entity {
        self.target
    }

    fn from(entity: Entity) -> Self {
        Self {
            next_action: None,
            target: entity,
        }
    }

    fn set_risky(&mut self, entity: Entity) {
        self.target = entity;
    }
}

#[derive(Component)]
#[relationship_target(relationship = GotoTarget, linked_spawn)]
pub struct TargetedBy(Vec<Entity>);

#[derive(Error, Debug)]
pub enum GotoError {
    #[error("goto target not found")]
    TargetNotFound,
    #[error("goto action not found")]
    NextActionNotFound,
    #[error("goto sensor state for sensor id {0:?} not found")]
    SensorStateNotFound(SensorId),
}

pub fn finish_goto(
    mut commands: Commands,
    query: Query<
        (Entity, &GotoTarget, &SensorState),
        Or<(Changed<GotoTarget>, Changed<SensorState>)>,
    >,
) -> Result {
    for (entity, goto_action, sensor_state) in &query {
        let target = goto_action
            .next_action()
            .ok_or(GotoError::NextActionNotFound)?
            .target()
            .as_ref()
            .ok_or(GotoError::TargetNotFound)?;
        let sensor_state = sensor_state
            .get(&target.id)
            .ok_or(GotoError::SensorStateNotFound(target.id))?;

        if sensor_state.is_close(target.value) {
            commands.entity(entity).insert_action(
                goto_action
                    .next_action()
                    .ok_or(GotoError::NextActionNotFound)?,
            );
        }
    }
    Ok(())
}
