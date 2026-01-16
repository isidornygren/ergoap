use bevy_ecs::{
    component::Component,
    entity::Entity,
    lifecycle::HookContext,
    reflect::ReflectCommandExt,
    system::{Commands, Query},
    world::DeferredWorld,
};
use bevy_reflect::PartialReflect;

use crate::{
    action_provider::ActionProviderTrait, astar::astar_plan, goal::Goal, sensor_state::SensorState,
};

pub fn on_insert_plan(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    if let Some(first_action) = world
        .get::<Plan>(entity)
        .and_then(|plan| plan.actions().first())
        .map(|action| action.to_dynamic())
    {
        world.commands().entity(entity).insert_reflect(first_action);
    }
}

#[derive(Component)]
#[component(on_insert=on_insert_plan)]
pub struct Plan(Vec<Box<dyn PartialReflect>>);

impl Plan {
    pub fn actions(&self) -> &Vec<Box<dyn PartialReflect>> {
        &self.0
    }
}

pub fn make_plan<'w>(
    mut commands: Commands,
    query: Query<(Entity, &SensorState, &dyn ActionProviderTrait, &Goal)>,
) {
    for (entity, sensor_values, actions, goal) in query.iter() {
        let dyn_actions: Vec<&dyn ActionProviderTrait> =
            actions.iter().map(|action| action.into_inner()).collect();
        if let Some(plan) = astar_plan(&sensor_values.to_owned(), dyn_actions, goal) {
            commands.entity(entity).insert(Plan(plan));
        }
    }
}
