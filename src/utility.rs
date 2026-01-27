use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use bevy_ecs::component::Component;
use bevy_trait_query::queryable;

use crate::{Comparison, Goal, IdContainer, goal::GoalBuilder};

pub struct Score(f32);

impl From<f32> for Score {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

pub trait Scorer {
    fn score(&self) -> Score;
}

#[queryable]
pub trait GoalProviderTrait: Send + Sync {
    fn score(&self) -> Score;
    fn goal(&self) -> &GoalBuilder;
}

#[derive(Component)]
pub struct GoalProvider<T: Scorer> {
    scorer: T,
    goal: GoalBuilder,
}

impl<T: Scorer + Send + Sync + 'static> GoalProvider<T> {
    pub fn from_requirement(scorer: T, requirement: IdContainer<TypeId, Comparison>) -> Self {
        Self {
            scorer,
            goal: Goal::from_requirement(requirement),
        }
    }
}

impl<T: Scorer> Deref for GoalProvider<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.scorer
    }
}

impl<T: Scorer> DerefMut for GoalProvider<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scorer
    }
}

impl<T: Scorer + Send + Sync + 'static> GoalProviderTrait for GoalProvider<T> {
    fn score(&self) -> Score {
        self.scorer.score()
    }

    fn goal(&self) -> &GoalBuilder {
        &self.goal
    }
}
