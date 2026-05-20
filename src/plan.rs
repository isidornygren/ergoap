use bevy_ecs::{
    component::Component,
    entity::Entity,
    lifecycle::HookContext,
    query::{Changed, Or},
    system::{Commands, Query},
    world::{DeferredWorld, Ref},
};

use crate::{
    Otherwise,
    action_provider::{ActionProvider, ActionProviders},
    astar::astar_plan,
    current_action::ActionCommands,
    goal::Goal,
    sensor_state::SensorState,
};

pub fn on_insert_plan(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    if let Some(first_action) = world
        .get_mut::<Plan>(entity)
        .and_then(|mut plan| plan.0.pop())
    {
        world
            .commands()
            .entity(entity)
            .insert_current_action(first_action.clone());
    }
}

#[derive(Component)]
#[component(on_insert=on_insert_plan)]
pub struct Plan(Vec<ActionProvider>);

pub fn make_plan(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &SensorState,
            &ActionProviders,
            &Goal,
            Option<&Otherwise>,
        ),
        Or<(Changed<SensorState>, Changed<Goal>)>,
    >,
) {
    for (entity, sensor_values, actions, goal, maybe_otherwise) in query.iter() {
        // let dyn_actions: Vec<&ActionProvider> = actions.iter().map(Ref::into_inner).collect();
        if let Some(plan) = astar_plan(&sensor_values.to_owned(), &actions.0, goal) {
            let plan_actions = plan
                .into_iter()
                .filter_map(|index| actions.0.get(index))
                .rev()
                .cloned()
                .collect();

            commands.entity(entity).insert(Plan(plan_actions));
        } else {
            commands.entity(entity).remove::<Plan>();
            if let Some(otherwise) = maybe_otherwise {
                commands
                    .entity(entity)
                    .insert_current_action(otherwise.action.clone());
            } else {
                commands.entity(entity).remove_current_action();
            }
        }
    }
}
