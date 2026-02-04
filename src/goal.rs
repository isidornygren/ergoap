use std::any::TypeId;

use bevy_ecs::{
    component::Component,
    lifecycle::HookContext,
    world::{DeferredWorld, EntityWorldMut},
};

use crate::{
    Comparison,
    id_container::{BuildSensorId, BuildSensorIdError, IdContainer},
    sensor_state::{SensorId, SensorState},
};

pub fn on_insert_goal_builder(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(goal_builder) = world.get::<GoalBuilder>(entity).cloned() {
        world
            .commands()
            .entity(entity)
            .queue(move |mut entity_world: EntityWorldMut| {
                let goal = goal_builder
                    .build(&mut entity_world)
                    .expect("Could not build goal");
                entity_world.insert(goal).remove_by_id(component_id);
            });
    }
}

#[derive(Component, Default, Clone)]
#[component(on_insert=on_insert_goal_builder)]
pub struct GoalBuilder {
    requirements: Vec<IdContainer<TypeId, Comparison>>,
}

impl GoalBuilder {
    pub fn build(&self, world: &mut EntityWorldMut) -> Result<Goal, BuildSensorIdError> {
        Ok(Goal {
            requirements: self
                .requirements
                .iter()
                .map(|requirement| world.build_sensor_container(*requirement))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn push_requirement(&mut self, requirement: IdContainer<TypeId, Comparison>) {
        self.requirements.push(requirement);
    }
}

#[derive(Component, Debug, Clone)]
pub struct Goal {
    requirements: Vec<IdContainer<SensorId, Comparison>>,
}

impl Goal {
    #[must_use]
    pub fn from_requirement(requirement: IdContainer<TypeId, Comparison>) -> GoalBuilder {
        GoalBuilder {
            requirements: vec![requirement],
        }
    }

    #[must_use]
    pub fn is_satisfied(&self, sensor_state: &SensorState) -> bool {
        self.requirements.iter().all(|IdContainer { id, value }| {
            sensor_state.get(id).is_some_and(|v| value.compare(*v))
        })
    }

    #[must_use]
    pub fn distance(&self, sensor_state: &SensorState) -> usize {
        self.requirements
            .iter()
            .filter(|IdContainer { id, value }| {
                sensor_state.get(id).is_none_or(|v| !value.compare(*v))
            })
            .count()
    }
}
