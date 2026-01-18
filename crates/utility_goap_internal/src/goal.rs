use std::any::TypeId;

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    world::{DeferredWorld, World},
};

use crate::{
    Comparison,
    id_container::{ComponentNotFound, IdContainer},
    sensor_state::SensorState,
};

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

#[derive(Component)]
#[component(on_insert=on_insert_goal_builder)]
pub struct GoalBuilder {
    requirements: Vec<IdContainer<TypeId, Comparison>>,
}

impl GoalBuilder {
    pub fn build(&self, world: &World) -> Result<Goal, ComponentNotFound> {
        Ok(Goal {
            requirements: self
                .requirements
                .iter()
                .map(|requirement| requirement.build(world))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Component, Debug)]
pub struct Goal {
    requirements: Vec<IdContainer<ComponentId, Comparison>>,
}

impl Goal {
    pub fn from_requirement(requirement: IdContainer<TypeId, Comparison>) -> GoalBuilder {
        GoalBuilder {
            requirements: vec![requirement],
        }
    }

    pub fn is_satisfied(&self, sensor_state: &SensorState) -> bool {
        self.requirements.iter().all(|IdContainer { id, value }| {
            sensor_state.get(id).map_or(false, |v| value.compare(*v))
        })
    }

    pub fn distance(&self, sensor_state: &SensorState) -> usize {
        self.requirements
            .iter()
            .filter(|IdContainer { id, value }| {
                sensor_state.get(id).map_or(true, |v| !value.compare(*v))
            })
            .count()
    }
}
