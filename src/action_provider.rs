//! Action providers define available actions and their preconditions/effects.
//!
//! This module provides the core trait and types for defining actions that entities can perform.
//! Actions have requirements (preconditions), effects on the world state, and costs for planning.

#[cfg(feature = "target")]
use std::any::Any;
use std::any::TypeId;

use bevy_ecs::{
    component::{Component, ComponentId, Immutable, StorageType},
    error::Result,
    lifecycle::HookContext,
    world::{DeferredWorld, EntityWorldMut, World},
};
use bevy_trait_query::queryable;

use crate::{
    Comparison, IdContainer,
    current_action::CurrentAction,
    effect::EffectValue,
    id_container::{BuildComponentId, ComponentNotFound},
    sensor_state::SensorState,
};

/// Trait for action providers that can be queried dynamically.
///
/// This trait defines the interface for actions in the GOAP system. Actions can check
/// if their preconditions are met, apply effects to sensor state, and be inserted as
/// the current action on an entity.
///
/// The trait is marked with `#[queryable]` to allow dynamic trait queries in Bevy.
#[queryable]
pub trait ActionProviderTrait: Send + Sync {
    /// Apply this action's effects to the given sensor state.
    ///
    /// This is used during planning to simulate the result of performing this action.
    fn apply(&self, sensor_values: &mut SensorState);

    /// Check if this action's preconditions are met given the current sensor state.
    ///
    /// Returns `true` if the action can be performed in the given state.
    fn preconditions_met(&self, _sensor_values: &SensorState) -> bool;

    /// Get the cost of performing this action.
    ///
    /// Lower costs are preferred during planning. The default cost is 1.
    fn cost(&self) -> usize;

    /// Insert the current action component on the entity.
    ///
    /// This method is called when the action is selected to be executed.
    fn insert_current_action(&self, entity_world: &mut EntityWorldMut);

    /// Clone this action provider into a boxed trait object.
    fn clone_box(&self) -> Box<dyn ActionProviderTrait>;

    /// Get the target component ID if this action requires a target (with `target` feature).
    #[cfg(feature = "target")]
    fn target(&self) -> &Option<ComponentId>;
}

/// Hook function called when an [`ActionProviderBuilder`] is inserted on an entity.
///
/// This automatically builds the action provider from the builder and replaces the builder
/// with the fully constructed [`ActionProvider`].
pub fn on_insert_action_provider_builder<C: Clone + Send + Sync + 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    world
        .commands()
        .entity(entity)
        .queue(|mut entity_world_mut: EntityWorldMut| -> Result {
            if let Some(action_provider_builder) =
                entity_world_mut.take::<ActionProviderBuilder<C>>()
            {
                let action_provider = action_provider_builder.build(entity_world_mut.world())?;
                entity_world_mut.insert(action_provider);
            }
            Ok(())
        });
}

/// Builder for constructing an [`ActionProvider`] with requirements, effects, cost, and target.
///
/// This type uses [`TypeId`] for component identification and must be converted to an
/// [`ActionProvider`] (which uses [`ComponentId`]) before use in planning. This conversion
/// happens automatically via the `on_insert` hook.
///
/// # Type Parameters
///
/// * `C` - The action type that will be stored in [`CurrentAction`] when this action executes
pub struct ActionProviderBuilder<C> {
    /// The action data to insert when this action is selected
    pub action: C,
    /// The planning cost of this action (lower is better)
    pub cost: usize,
    /// Preconditions that must be met for this action to be available
    pub requirements: Vec<IdContainer<TypeId, Comparison>>,
    /// Effects this action has on the world state
    pub effects: Vec<IdContainer<TypeId, EffectValue>>,
    /// Target component type required by this action (with `target` feature)
    #[cfg(feature = "target")]
    pub target: Option<TypeId>,
}

impl<C> ActionProviderBuilder<C> {
    /// Set the cost of this action for planning purposes.
    ///
    /// Lower costs are preferred by the planner. Default is 1.
    #[must_use]
    pub const fn with_cost(mut self, cost: usize) -> Self {
        self.cost = cost;
        self
    }

    /// Add an effect that this action has on the world state.
    ///
    /// Effects are applied during planning to simulate the result of this action.
    #[must_use]
    pub fn with_effect(mut self, effect: IdContainer<TypeId, EffectValue>) -> Self {
        self.effects.push(effect);
        self
    }

    /// Add a precondition requirement for this action.
    ///
    /// All requirements must be satisfied for the action to be available.
    #[must_use]
    pub fn with_requirement(mut self, requirement: IdContainer<TypeId, Comparison>) -> Self {
        self.requirements.push(requirement);
        self
    }

    /// Set the target component type that this action requires (with `target` feature).
    ///
    /// The action will only be available when a valid target exists in the sensor state.
    #[cfg(feature = "target")]
    #[must_use]
    pub const fn with_target<T: Any>(mut self) -> Self {
        self.target = Some(TypeId::of::<T>());
        self
    }
}

impl<C: Clone + Send + Sync + 'static> Component for ActionProviderBuilder<C> {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    type Mutability = Immutable;

    fn on_insert() -> Option<bevy_ecs::lifecycle::ComponentHook> {
        Some(on_insert_action_provider_builder::<C>)
    }
}

impl<C> ActionProviderBuilder<C> {
    pub(crate) fn build(self, world: &World) -> Result<ActionProvider<C>, ComponentNotFound> {
        Ok(ActionProvider {
            cost: self.cost,
            action: self.action,
            requirements: self
                .requirements
                .into_iter()
                .map(|requirement| world.build_id_container(requirement))
                .collect::<Result<Vec<_>, _>>()?,
            effects: self
                .effects
                .into_iter()
                .map(|effect| world.build_id_container(effect))
                .collect::<Result<Vec<_>, _>>()?,
            #[cfg(feature = "target")]
            target: match &self.target {
                Some(target) => Some(world.get_component_id(target)?),
                None => None,
            },
        })
    }
}

/// A concrete action provider component with resolved component IDs.
///
/// This type is the result of building an [`ActionProviderBuilder`] and uses [`ComponentId`]
/// for efficient runtime lookups. It stores the action data and all metadata needed for planning.
///
/// # Type Parameters
///
/// * `C` - The action type that will be cloned into [`CurrentAction`] when executed
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// #[derive(Clone)]
/// struct MoveAction {
///     speed: f32,
/// }
///
/// // Create an action provider
/// let provider = ActionProvider::new(MoveAction { speed: 5.0 })
///     .with_cost(2)
///     .with_requirement(HasEnergy::greater_than(10))
///     .with_effect(AtDestination::set(true));
/// ```
#[derive(Component, Clone)]
#[require(SensorState)]
pub struct ActionProvider<C> {
    /// The action data to clone when this action executes
    pub action: C,
    /// The planning cost (lower is preferred)
    pub cost: usize,
    /// Preconditions for availability
    pub requirements: Vec<IdContainer<ComponentId, Comparison>>,
    /// Effects on world state
    pub effects: Vec<IdContainer<ComponentId, EffectValue>>,
    /// Required target component (with `target` feature)
    #[cfg(feature = "target")]
    pub target: Option<ComponentId>,
}

impl<C> ActionProvider<C> {
    /// Create a new action provider builder.
    ///
    /// Returns an [`ActionProviderBuilder`] that can be configured with requirements,
    /// effects, cost, and target before being inserted on an entity.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let action = ActionProvider::new(MyAction::default())
    ///     .with_cost(5)
    ///     .with_requirement(MySensor::is_true());
    /// ```
    pub const fn new(action: C) -> ActionProviderBuilder<C> {
        ActionProviderBuilder {
            action,
            cost: 1,
            requirements: vec![],
            effects: vec![],
            #[cfg(feature = "target")]
            target: None,
        }
    }
}

impl<C: Clone + Send + Sync + 'static> ActionProviderTrait for ActionProvider<C> {
    fn apply(&self, sensor_values: &mut SensorState) {
        for IdContainer {
            id,
            value: effect_value,
        } in &self.effects
        {
            match effect_value {
                EffectValue::Set(value) => sensor_values.insert(*id, *value),
            }
        }
    }

    fn preconditions_met(&self, sensor_values: &SensorState) -> bool {
        #[cfg(feature = "target")]
        if let Some(target) = &self.target
            && sensor_values.get(target).is_none_or(|v| !v.has_target())
        {
            return false;
        }
        self.requirements.iter().all(|IdContainer { id, value }| {
            sensor_values.get(id).is_some_and(|v| value.compare(v))
        })
    }

    fn cost(&self) -> usize {
        self.cost
    }

    fn insert_current_action(&self, entity_world: &mut EntityWorldMut) {
        entity_world.insert(CurrentAction {
            action: self.action.clone(),
        });
    }

    fn clone_box(&self) -> Box<dyn ActionProviderTrait> {
        Box::new(self.clone())
    }

    #[cfg(feature = "target")]
    fn target(&self) -> &Option<ComponentId> {
        &self.target
    }
}
