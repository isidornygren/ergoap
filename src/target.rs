//! Target-based movement and action system.
//!
//! This module provides the `GotoTarget` action for automatically moving entities
//! toward targets before executing actions that require proximity. It integrates
//! with the sensor system to detect when targets are close enough.

use bevy_ecs::{
    component::{Component, ComponentId},
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

/// Internal data for the goto target action.
///
/// This struct stores the target entity and the next action to execute once
/// the target is reached. It is wrapped by [`CurrentAction<GotoTarget>`].
pub struct GotoTarget {
    /// The action to execute once the target is reached
    pub(crate) next_action: Option<Box<dyn ActionProviderTrait>>,
    /// The target entity to move toward
    pub(crate) target: Entity,
}

/// Extension methods for [`CurrentAction<GotoTarget>`].
///
/// Provides convenient accessors for the target entity and next action.
impl CurrentAction<GotoTarget> {
    /// Get the target entity this goto action is moving toward.
    ///
    /// # Returns
    ///
    /// The entity being targeted
    #[must_use]
    pub const fn target(&self) -> Entity {
        self.action.target
    }
    /// Get the next action to execute once the target is reached.
    ///
    /// # Returns
    ///
    /// * `Some(&dyn ActionProviderTrait)` - The action to execute at the target
    /// * `None` - If no follow-up action is configured
    #[must_use]
    pub fn next_action(&self) -> Option<&dyn ActionProviderTrait> {
        self.action.next_action.as_deref()
    }
}

/// Relationship implementation for tracking which entities are targeted.
///
/// This allows querying for all entities targeting a specific entity using
/// Bevy's relationship system.
impl Relationship for CurrentAction<GotoTarget> {
    /// The relationship target component type
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
        self.action.target = entity;
    }
}

/// Component tracking which entities are targeting this entity.
///
/// This is automatically managed by Bevy's relationship system and updated
/// when [`CurrentAction<GotoTarget>`] components are added or removed.
#[derive(Component)]
#[relationship_target(relationship = CurrentAction<GotoTarget>, linked_spawn)]
pub struct TargetedBy(Vec<Entity>);

/// Errors that can occur during goto target processing.
#[derive(Error, Debug)]
pub enum GotoError {
    /// The target entity or target component was not found
    #[error("goto target not found")]
    TargetNotFound,
    /// No follow-up action was configured for the goto
    #[error("goto action not found")]
    NextActionNotFound,
    /// The sensor state doesn't contain the required target component
    #[error("goto sensor state for component id {0:?} not found")]
    SensorStateNotFound(ComponentId),
}

/// System that transitions from goto actions to their follow-up actions when targets are reached.
///
/// This system monitors entities with [`CurrentAction<GotoTarget>`] and checks if they've
/// reached their targets. When the target sensor indicates the entity is close enough,
/// it replaces the goto action with the intended follow-up action.
///
/// # Behavior
///
/// - Runs in `FixedPostUpdate` schedule
/// - Only processes entities where goto action or sensor state changed
/// - Checks if the target's `is_close` flag is true in the sensor state
/// - Automatically inserts the next action when the target is reached
///
/// # Errors
///
/// Returns an error if:
/// - The next action is not configured
/// - The target sensor component is not found
/// - The sensor state doesn't contain the target value
pub fn finish_goto(
    mut commands: Commands,
    query: Query<
        (Entity, &CurrentAction<GotoTarget>, &SensorState),
        Or<(Changed<CurrentAction<GotoTarget>>, Changed<SensorState>)>,
    >,
) -> Result {
    for (entity, goto_action, sensor_state) in &query {
        let sensor_component_id = goto_action
            .next_action()
            .ok_or(GotoError::NextActionNotFound)?
            .target()
            .as_ref()
            .ok_or(GotoError::TargetNotFound)?;
        let sensor_state = sensor_state
            .get(sensor_component_id)
            .ok_or(GotoError::SensorStateNotFound(*sensor_component_id))?;
        if let SensorValue::Target(Some(TargetValue { is_close, .. })) = sensor_state
            && *is_close
        {
            commands.entity(entity).insert_action(
                goto_action
                    .next_action()
                    .ok_or(GotoError::NextActionNotFound)?,
            );
        }
    }
    Ok(())
}
