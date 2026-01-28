use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query},
    world::Ref,
};

use crate::{Goal, GoalProviderTrait, Score, SensorState};

#[derive(Component)]
pub struct Picker {
    pick: Box<
        dyn Fn(&mut dyn Iterator<Item = Ref<'_, dyn GoalProviderTrait>>) -> Option<Goal>
            + Sync
            + Send,
    >,
}

fn highest_scorer_pick(
    choices: &mut dyn Iterator<Item = Ref<'_, dyn GoalProviderTrait>>,
) -> Option<Goal> {
    choices
        .max_by_key(|provider| provider.score())
        .map(|provider| provider.goal().clone())
}

fn first_to_score_pick(
    choices: &mut dyn Iterator<Item = Ref<'_, dyn GoalProviderTrait>>,
    threshold: Score,
) -> Option<Goal> {
    for choice in choices {
        if choice.score() > threshold {
            return Some(choice.goal().clone());
        }
    }
    None
}

impl Picker {
    fn pick(
        &self,
        choices: &mut dyn Iterator<Item = Ref<'_, dyn GoalProviderTrait>>,
    ) -> Option<Goal> {
        (self.pick)(choices)
    }

    #[must_use]
    pub fn from_fn(
        f: impl Fn(&mut dyn Iterator<Item = Ref<'_, dyn GoalProviderTrait>>) -> Option<Goal>
        + Sync
        + Send
        + 'static,
    ) -> Self {
        Self { pick: Box::new(f) }
    }

    #[must_use]
    pub fn highest_scorer() -> Self {
        Self::from_fn(highest_scorer_pick)
    }

    #[must_use]
    pub fn first_to_score(threshold: Score) -> Self {
        Self::from_fn(move |choices| first_to_score_pick(choices, threshold))
    }
}

pub fn picking_system(
    mut commands: Commands,
    query: Query<(Entity, &Picker, &dyn GoalProviderTrait, &SensorState)>,
) {
    for (entity, current_picker, goal_providers, sensor_state) in query.iter() {
        let mut viable_goals = goal_providers
            .iter()
            .filter(|goal_provider| !goal_provider.goal().is_satisfied(sensor_state));
        if let Some(next_goal) = current_picker.pick(&mut viable_goals) {
            commands.entity(entity).insert(next_goal);
        }
    }
}
