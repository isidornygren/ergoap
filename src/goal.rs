use std::any::TypeId;

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    world::DeferredWorld,
};

use crate::{requirement::Requirement, sensor_state::SensorState};

pub fn on_insert_goal_builder(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(goal_builder) = world.get::<GoalBuilder>(entity) {
        let goal = goal_builder.build(&world);
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
    requirements: Vec<Requirement<TypeId>>,
}

impl GoalBuilder {
    pub fn build(&self, world: &DeferredWorld) -> Goal {
        Goal {
            requirements: self
                .requirements
                .iter()
                .map(|Requirement { id, comparison }| Requirement {
                    comparison: comparison.to_owned(),
                    id: world
                        .components()
                        .get_id(*id)
                        .expect("Could not get component id"),
                })
                .collect(),
        }
    }
}

#[derive(Component)]
pub struct Goal {
    requirements: Vec<Requirement<ComponentId>>,
}

impl Goal {
    pub fn from_requirement(requirement: Requirement<TypeId>) -> GoalBuilder {
        GoalBuilder {
            requirements: vec![requirement],
        }
    }

    pub fn is_satisfied(&self, sensor_state: &SensorState) -> bool {
        self.requirements
            .iter()
            .all(|Requirement { id, comparison }| {
                sensor_state
                    .get(id)
                    .map_or(false, |v| comparison.compare(*v))
            })
    }

    pub fn distance(&self, sensor_state: &SensorState) -> usize {
        self.requirements
            .iter()
            .filter(|Requirement { id, comparison }| {
                sensor_state
                    .get(id)
                    .map_or(true, |v| !comparison.compare(*v))
            })
            .count()
    }
}
