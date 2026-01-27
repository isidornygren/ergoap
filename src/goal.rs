//! Goal definitions and requirements for GOAP planning.
//!
//! This module provides types for defining goals that the planner tries to achieve.
//! Goals consist of requirements on sensor values that must be satisfied for the goal
//! to be considered complete.

use std::any::TypeId;

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    world::{DeferredWorld, World},
};

use crate::{
    Comparison,
    id_container::{BuildComponentId, ComponentNotFound, IdContainer},
    sensor_state::SensorState,
};

/// Hook function called when a [`GoalBuilder`] is inserted.
///
/// This automatically builds the goal from the builder, resolving [`TypeId`] references
/// to [`ComponentId`] references, and replaces the builder with the constructed [`Goal`].
pub fn on_insert_goal_builder(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(goal_builder) = world.get::<GoalBuilder>(entity) {
        let goal = goal_builder.build(&world).expect("Could not build goal");
        world
            .commands()
            .entity(entity)
            .insert(goal)
            .remove_by_id(component_id);
    }
}

/// Builder for constructing a [`Goal`] with type-based requirements.
///
/// This builder uses [`TypeId`] for component identification and must be converted
/// to a [`Goal`] (which uses [`ComponentId`]) before use in planning. The conversion
/// happens automatically via the `on_insert` hook.
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// // Create a goal requiring a sensor to be true
/// let goal = Goal::from_requirement(MySensor::is_true());
/// ```
#[derive(Component)]
#[component(on_insert=on_insert_goal_builder)]
pub struct GoalBuilder {
    requirements: Vec<IdContainer<TypeId, Comparison>>,
}

impl GoalBuilder {
    /// Build the goal by resolving type IDs to component IDs.
    ///
    /// # Errors
    ///
    /// Returns [`ComponentNotFound`] if any requirement references a type that
    /// hasn't been registered with the world.
    pub fn build(&self, world: &World) -> Result<Goal, ComponentNotFound> {
        Ok(Goal {
            requirements: self
                .requirements
                .iter()
                .map(|requirement| world.build_id_container(*requirement))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// A goal that the planner tries to achieve.
///
/// Goals consist of a set of requirements on sensor values. The planner uses
/// A* search to find action sequences that satisfy all requirements.
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// // Create a goal with a single requirement
/// commands.entity(entity).insert(
///     Goal::from_requirement(HasFood::is_true())
/// );
/// ```
#[derive(Component, Debug)]
pub struct Goal {
    requirements: Vec<IdContainer<ComponentId, Comparison>>,
}

impl Goal {
    /// Create a new goal builder with a single requirement.
    ///
    /// Use sensor comparison methods to create requirements:
    ///
    /// # Example
    ///
    /// ```ignore
    /// let goal = Goal::from_requirement(MySensor::equal(true));
    /// ```
    #[must_use]
    pub fn from_requirement(requirement: IdContainer<TypeId, Comparison>) -> GoalBuilder {
        GoalBuilder {
            requirements: vec![requirement],
        }
    }

    /// Check if this goal is satisfied by the given sensor state.
    ///
    /// Returns `true` if all requirements are met, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `sensor_state` - The current sensor state to check against
    #[must_use]
    pub fn is_satisfied(&self, sensor_state: &SensorState) -> bool {
        self.requirements
            .iter()
            .all(|IdContainer { id, value }| sensor_state.get(id).is_some_and(|v| value.compare(v)))
    }

    /// Calculate the heuristic distance from the sensor state to this goal.
    ///
    /// Returns the number of unsatisfied requirements. This is used as the
    /// heuristic function in A* planning.
    ///
    /// # Arguments
    ///
    /// * `sensor_state` - The sensor state to calculate distance from
    ///
    /// # Returns
    ///
    /// The count of requirements that are not satisfied
    #[must_use]
    pub fn distance(&self, sensor_state: &SensorState) -> usize {
        self.requirements
            .iter()
            .filter(|IdContainer { id, value }| {
                sensor_state.get(id).is_none_or(|v| !value.compare(v))
            })
            .count()
    }
}
