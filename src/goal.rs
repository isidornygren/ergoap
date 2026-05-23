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
        world.commands().entity(entity).queue(
            move |mut entity_world: EntityWorldMut| -> Result<(), BuildSensorIdError> {
                let goal = goal_builder.build(&mut entity_world)?;
                entity_world.insert(goal).remove_by_id(component_id);
                Ok(())
            },
        );
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::test_utils::*;

    #[test]
    fn goal_builder_produces_goal_on_spawn() {
        let mut test_app = setup_test_app();
        let entity = test_app
            .world_mut()
            .spawn((
                SensorState::default(),
                TestSensor(true),
                Goal::from_requirement(TestSensor::is_true()),
            ))
            .id();

        let entity = test_app.world().entity(entity);
        let components = entity.archetype().components();

        let expected = [
            test_app.get_component_id::<SensorState>(),
            test_app.get_component_id::<TestSensor>(),
            test_app.get_component_id::<Goal>(),
        ];

        assert_eq!(components.len(), expected.len());
        for id in &expected {
            assert!(components.contains(id), "Missing component {id:?}");
        }
    }

    #[test]
    fn goal_is_satisfied() {
        let mut test_app = setup_test_app();
        let entity = test_app
            .world_mut()
            .spawn((
                SensorState::default(),
                TestSensor(true),
                Goal::from_requirement(TestSensor::is_true()),
            ))
            .id();

        test_app.world_mut().run_schedule(Planning);

        let entity = test_app.world().entity(entity);
        let goal = entity.get::<Goal>().unwrap();
        let sensor_state = entity.get::<SensorState>().unwrap();

        assert!(goal.is_satisfied(sensor_state));
    }

    #[test]
    fn goal_is_not_satisfied() {
        let mut test_app = setup_test_app();
        let entity = test_app
            .world_mut()
            .spawn((
                SensorState::default(),
                TestSensor(false),
                Goal::from_requirement(TestSensor::is_true()),
            ))
            .id();

        test_app.world_mut().run_schedule(Planning);

        let entity = test_app.world().entity(entity);
        let goal = entity.get::<Goal>().unwrap();
        let sensor_state = entity.get::<SensorState>().unwrap();

        assert!(!goal.is_satisfied(sensor_state));
    }
}
