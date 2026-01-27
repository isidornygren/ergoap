//! Utility-based goal scoring system.
//!
//! This module provides types and traits for scoring goals based on their utility.
//! Higher-scoring goals are prioritized by the planning system, allowing entities
//! to dynamically choose between multiple possible objectives.

use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use bevy_ecs::component::Component;
use bevy_trait_query::queryable;

use crate::{Comparison, Goal, IdContainer, goal::GoalBuilder};

/// A utility score representing the desirability of a goal.
///
/// Higher scores indicate more desirable goals. The planning system can use
/// scores to prioritize between multiple possible goals.
///
/// # Example
///
/// ```ignore
/// use utility_goap::Score;
///
/// let score = Score::from(0.8);
/// ```
pub struct Score(f32);

impl From<f32> for Score {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

/// Trait for types that can calculate a utility score.
///
/// Implement this trait on types that need to evaluate how desirable
/// a goal is based on the current game state.
///
/// # Example
///
/// ```ignore
/// use utility_goap::{Score, Scorer};
///
/// struct HungerScorer {
///     hunger_level: f32,
/// }
///
/// impl Scorer for HungerScorer {
///     fn score(&self) -> Score {
///         Score::from(self.hunger_level)
///     }
/// }
/// ```
pub trait Scorer {
    /// Calculate the utility score for this scorer.
    ///
    /// # Returns
    ///
    /// A [`Score`] representing how desirable the associated goal is
    fn score(&self) -> Score;
}

/// Trait for goal providers that can be queried dynamically.
///
/// This trait combines scoring with goal definition, allowing the planning system
/// to query available goals and their scores. The trait is marked with `#[queryable]`
/// to enable dynamic trait queries in Bevy.
#[queryable]
pub trait GoalProviderTrait: Send + Sync {
    /// Calculate the utility score for this goal.
    ///
    /// # Returns
    ///
    /// A [`Score`] representing how desirable this goal is
    fn score(&self) -> Score;

    /// Get the goal definition with its requirements.
    ///
    /// # Returns
    ///
    /// A reference to the goal builder that defines this goal's requirements
    fn goal(&self) -> &GoalBuilder;
}

/// A component that provides a goal with utility-based scoring.
///
/// This component wraps a [`Scorer`] implementation and associates it with a goal.
/// The planning system can query these components to find and prioritize goals
/// based on their utility scores.
///
/// # Type Parameters
///
/// * `T` - A type implementing [`Scorer`] that calculates the goal's utility
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// #[derive(Component)]
/// struct HungerScorer {
///     hunger: f32,
/// }
///
/// impl Scorer for HungerScorer {
///     fn score(&self) -> Score {
///         Score::from(self.hunger)
///     }
/// }
///
/// // Create a goal provider for finding food when hungry
/// let goal_provider = GoalProvider::from_requirement(
///     HungerScorer { hunger: 0.8 },
///     HasFood::is_true()
/// );
/// ```
#[derive(Component)]
pub struct GoalProvider<T: Scorer> {
    scorer: T,
    goal: GoalBuilder,
}

impl<T: Scorer + Send + Sync + 'static> GoalProvider<T> {
    /// Create a new goal provider with a scorer and a single requirement.
    ///
    /// # Arguments
    ///
    /// * `scorer` - The scorer that calculates utility for this goal
    /// * `requirement` - The sensor requirement that defines this goal
    ///
    /// # Example
    ///
    /// ```ignore
    /// let provider = GoalProvider::from_requirement(
    ///     MyScorer::default(),
    ///     MySensor::is_true()
    /// );
    /// ```
    pub fn from_requirement(scorer: T, requirement: IdContainer<TypeId, Comparison>) -> Self {
        Self {
            scorer,
            goal: Goal::from_requirement(requirement),
        }
    }
}

/// Allow direct access to the scorer through the goal provider.
impl<T: Scorer> Deref for GoalProvider<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.scorer
    }
}

/// Allow mutable access to the scorer through the goal provider.
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
