use bevy_ecs::{
    component::Component,
    entity::Entity,
    lifecycle::HookContext,
    query::{Changed, Or},
    system::{Commands, ParallelCommands, Query},
    world::{DeferredWorld, Ref},
};

use crate::{
    Otherwise, action_provider::ActionProviderTrait, astar::astar_plan,
    current_action::CurrentActionCommands, goal::Goal, sensor_state::SensorState,
};

pub fn on_insert_plan(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    if let Some(first_action) = world
        .get_mut::<Plan>(entity)
        .and_then(|mut plan| plan.0.pop())
    {
        world
            .commands()
            .entity(entity)
            .spawn_current_action(first_action);
    }
}

#[derive(Component)]
#[component(on_insert=on_insert_plan)]
pub struct Plan(Vec<Box<dyn ActionProviderTrait>>);

pub fn make_plan(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &SensorState,
            &dyn ActionProviderTrait,
            &Goal,
            Option<&Otherwise>,
        ),
        Or<(Changed<SensorState>, Changed<Goal>)>,
    >,
) {
    for (entity, sensor_values, actions, goal, maybe_otherwise) in query.iter() {
        let dyn_actions: Vec<&dyn ActionProviderTrait> =
            actions.iter().map(Ref::into_inner).collect();
        if let Some(plan) = astar_plan(&sensor_values.to_owned(), &dyn_actions, goal) {
            let plan_actions = plan
                .into_iter()
                .filter_map(|index| dyn_actions.get(index))
                .map(|action| action.clone_box())
                .rev()
                .collect();

            commands.entity(entity).insert(Plan(plan_actions));
        } else {
            commands.entity(entity).remove::<Plan>();
            if let Some(otherwise) = maybe_otherwise {
                commands
                    .entity(entity)
                    .spawn_current_action(otherwise.action.clone_box());
            } else {
                commands.entity(entity).despawn_current_action();
            }
        }
    }
}
