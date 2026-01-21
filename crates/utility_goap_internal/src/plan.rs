use bevy_ecs::{
    component::Component,
    entity::Entity,
    lifecycle::HookContext,
    query::Changed,
    reflect::ReflectCommandExt,
    system::{Commands, Query},
    world::DeferredWorld,
};
use bevy_reflect::PartialReflect;

use crate::{
    action_provider::ActionProviderTrait, astar::astar_plan, current_action::CurrentActionCommands,
    goal::Goal, sensor_state::SensorState,
};

pub fn on_insert_plan(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    if let Some(first_action) = world
        .get_mut::<Plan>(entity)
        .and_then(|mut plan| plan.0.pop())
        .map(|action| action.to_dynamic())
    {
        world.commands().entity(entity).insert_reflect(first_action);
    }
}

#[derive(Component)]
#[component(on_insert=on_insert_plan)]
pub struct Plan(Vec<Box<dyn PartialReflect>>);

pub fn make_plan<'w>(
    mut commands: Commands,
    query: Query<(Entity, &SensorState, &dyn ActionProviderTrait, &Goal), Changed<SensorState>>,
) {
    for (entity, sensor_values, actions, goal) in query.iter() {
        let dyn_actions: Vec<&dyn ActionProviderTrait> =
            actions.iter().map(|action| action.into_inner()).collect();
        if let Some(plan) = astar_plan(&sensor_values.to_owned(), &dyn_actions, goal) {
            println!("Plan ({:?}): {:?}", entity, plan);
            let plan_actions = plan
                .into_iter()
                .filter_map(|index| dyn_actions.get(index))
                .map(|action| action.component().to_dynamic())
                .rev()
                .collect();

            commands.entity(entity).insert(Plan(plan_actions));
        } else {
            println!("No plan, remove plan: {:?}", entity);
            commands
                .entity(entity)
                .remove::<Plan>()
                .despawn_current_action();
        }
    }
}
