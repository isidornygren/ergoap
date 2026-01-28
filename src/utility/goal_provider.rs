use bevy_ecs::{
    component::{Component, Immutable, StorageType},
    error::Result,
    lifecycle::HookContext,
    world::{DeferredWorld, EntityWorldMut, World},
};
use bevy_trait_query::queryable;
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use crate::{
    Comparison, Goal, IdContainer, Score, Scorer, goal::GoalBuilder,
    id_container::ComponentNotFound,
};

#[queryable]
pub trait GoalProviderTrait: Send + Sync {
    fn score(&self) -> Score;
    fn goal(&self) -> &Goal;
}

#[derive(Component)]
pub struct GoalProvider<T: Scorer> {
    scorer: T,
    goal: Goal,
}

pub struct GoalProviderBuilder<T: Scorer> {
    scorer: T,
    goal: GoalBuilder,
}

impl<T: Scorer> GoalProviderBuilder<T> {
    pub(crate) fn build(self, world: &World) -> Result<GoalProvider<T>, ComponentNotFound> {
        Ok(GoalProvider {
            scorer: self.scorer,
            goal: self.goal.build(world)?,
        })
    }

    #[must_use]
    pub fn with_requirement(mut self, requirement: IdContainer<TypeId, Comparison>) -> Self {
        self.goal.push_requirement(requirement);
        self
    }
}

pub fn on_insert_goal_provider_builder<T: Scorer + Send + Sync + 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    world
        .commands()
        .entity(entity)
        .queue(|mut entity_world_mut: EntityWorldMut| -> Result {
            if let Some(goal_provider_builder) = entity_world_mut.take::<GoalProviderBuilder<T>>() {
                let action_provider = goal_provider_builder.build(entity_world_mut.world())?;
                entity_world_mut.insert(action_provider);
            }
            Ok(())
        });
}

impl<T: Scorer + Send + Sync + 'static> Component for GoalProviderBuilder<T> {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    type Mutability = Immutable;

    fn on_insert() -> Option<bevy_ecs::lifecycle::ComponentHook> {
        Some(on_insert_goal_provider_builder::<T>)
    }
}

impl<T: Scorer + Send + Sync + 'static + Default> GoalProvider<T> {
    #[must_use]
    pub fn new(scorer: T) -> GoalProviderBuilder<T> {
        GoalProviderBuilder {
            scorer,
            goal: GoalBuilder::default(),
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

    fn goal(&self) -> &Goal {
        &self.goal
    }
}
