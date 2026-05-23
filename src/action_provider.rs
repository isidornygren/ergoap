#[cfg(feature = "target")]
use std::any::Any;
use std::any::TypeId;

use bevy_ecs::{
    component::{Component, Immutable, StorageType},
    error::Result,
    lifecycle::HookContext,
    world::{DeferredWorld, EntityWorldMut},
};
use bevy_trait_query::queryable;
use bitvec::vec::BitVec;

use crate::{
    Comparison, IdContainer, SensorValue,
    current_action::{CurrentActionRef, UpdateActionRef},
    id_container::{BuildSensorId, BuildSensorIdError},
    sensor_state::{SensorId, SensorState},
};
#[cfg(feature = "target")]
use crate::{TargetValue, WorldSensorValue};

#[queryable]
pub trait ActionProviderTrait: Send + Sync {
    fn apply(&self, sensor_values: &mut SensorState);
    fn apply_to_bitvec(&self, bitvec: &mut BitVec);
    fn preconditions_met(&self, _sensor_values: &SensorState) -> bool;
    fn cost(&self) -> usize;
    fn add_to_entity_world(&self, entity_world: &mut EntityWorldMut);
    fn clone_box(&self) -> Box<dyn ActionProviderTrait>;
    #[cfg(feature = "target")]
    fn target(&self) -> &Option<IdContainer<SensorId, f32>>;
}

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
                let action_provider = action_provider_builder.build(&mut entity_world_mut)?;
                entity_world_mut.insert(action_provider);
            }
            Ok(())
        });
}

pub struct ActionProviderBuilder<C> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<IdContainer<TypeId, Comparison>>,
    pub effects: Vec<IdContainer<TypeId, SensorValue>>,
    #[cfg(feature = "target")]
    pub target: Option<IdContainer<TypeId, f32>>,
}

impl<C> ActionProviderBuilder<C> {
    #[must_use]
    pub const fn with_cost(mut self, cost: usize) -> Self {
        self.cost = cost;
        self
    }

    #[must_use]
    pub fn with_effect(mut self, effect: IdContainer<TypeId, SensorValue>) -> Self {
        self.effects.push(effect);
        self
    }

    #[must_use]
    pub fn with_requirement(mut self, requirement: IdContainer<TypeId, Comparison>) -> Self {
        self.requirements.push(requirement);
        self
    }

    #[cfg(feature = "target")]
    #[must_use]
    pub const fn with_target<T: Any + WorldSensorValue<Option<TargetValue>>>(
        mut self,
        distance: f32,
    ) -> Self {
        self.target = Some(IdContainer {
            id: TypeId::of::<T>(),
            value: distance,
        });
        self
    }
}

impl<C: Clone + Send + Sync + 'static> Component for ActionProviderBuilder<C> {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    type Mutability = Immutable;

    fn on_insert() -> Option<bevy_ecs::lifecycle::ComponentHook> {
        Some(on_insert_action_provider_builder::<C>)
    }

    fn register_required_components(
        _component_id: bevy_ecs::component::ComponentId,
        required_components: &mut bevy_ecs::component::RequiredComponentsRegistrator,
    ) {
        required_components.register_required(SensorState::default);
    }
}

impl<C> ActionProviderBuilder<C> {
    pub(crate) fn build(
        self,
        world: &mut EntityWorldMut,
    ) -> Result<ActionProvider<C>, BuildSensorIdError> {
        Ok(ActionProvider {
            cost: self.cost,
            action: self.action,
            requirements: self
                .requirements
                .into_iter()
                .map(|requirement| world.build_sensor_container(requirement))
                .collect::<Result<Vec<_>, _>>()?,
            effects: self
                .effects
                .into_iter()
                .map(|effect| world.build_sensor_container(effect))
                .collect::<Result<Vec<_>, _>>()?,
            #[cfg(feature = "target")]
            target: self
                .target
                .map(|target| world.build_sensor_container(target))
                .transpose()?,
        })
    }
}

/// An ``ActionProvider`` provides the planner with actions that can be selected and executed.
/// It returns an ``ActionProviderBuilder`` which can be inserted in an entity.
///
/// # Example
/// ```rust
/// # use ergoap::prelude::*;
/// # use bevy::prelude::*;
///
/// #[derive(WorldSensor, Component)]
/// struct SomeSensor(bool);
///
/// #[derive(Clone)]
/// struct SomeAction;
///
/// let action_provider = ActionProvider::new(SomeAction)
///    .with_requirement(SomeSensor::is_false())
///    .with_effect(SomeSensor::set(true))
///    .with_cost(2);
///
/// assert_eq!(action_provider.requirements, vec![SomeSensor::is_false()]);
/// assert_eq!(action_provider.effects, vec![SomeSensor::set(true)]);
/// assert_eq!(action_provider.cost, 2);
/// ```
#[derive(Component, Clone)]
pub struct ActionProvider<C> {
    /// The action that will spawn in the entity when the planner has chosen this ``ActionProvider``.
    pub action: C,
    /// The cost of the action, the higher the cost, the more expensive the action.
    pub cost: usize,
    /// The state requirements for this action to be valid.
    /// If _all_ requirements are not met, the action cannot be selected.
    pub requirements: Vec<IdContainer<SensorId, Comparison>>,
    /// The state effects of this action.
    pub effects: Vec<IdContainer<SensorId, SensorValue>>,
    #[cfg(feature = "target")]
    /// The target of the action, if the ``TargetValue`` does not have an entity it will not be able to be chosen.
    /// If the ``TargetValue``s ``is_close`` field is not ``true``, it will spawn a ``CurrentAction<GotoTarget>``.
    pub target: Option<IdContainer<SensorId, f32>>,
}

impl<C> ActionProvider<C> {
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

impl<C: Component + Clone> ActionProviderTrait for ActionProvider<C> {
    fn apply(&self, sensor_values: &mut SensorState) {
        for IdContainer { id, value } in &self.effects {
            sensor_values.insert(*id, *value);
        }
    }

    fn apply_to_bitvec(&self, bitvec: &mut BitVec) {
        for IdContainer { id, value } in &self.effects {
            bitvec.set(id.0, value.as_bool());
        }
    }

    fn preconditions_met(&self, sensor_values: &SensorState) -> bool {
        #[cfg(feature = "target")]
        if let Some(target) = &self.target
            && sensor_values
                .get(&target.id)
                .is_none_or(|v| !v.has_target())
        {
            return false;
        }
        self.requirements.iter().all(|IdContainer { id, value }| {
            sensor_values.get(id).is_some_and(|v| value.compare(*v))
        })
    }

    fn cost(&self) -> usize {
        self.cost
    }

    fn add_to_entity_world(&self, entity_world: &mut EntityWorldMut) {
        if let Some(component_id) = entity_world.world_scope(|w| w.component_id::<C>()) {
            entity_world.update_action_ref(component_id);
        } else {
            // This should probably not happen, but adding it here as a failsafe if a component is not registered.
            entity_world.remove::<CurrentActionRef>();
        }
        entity_world.insert(self.action.clone());
    }

    fn clone_box(&self) -> Box<dyn ActionProviderTrait> {
        Box::new(self.clone())
    }

    #[cfg(feature = "target")]
    fn target(&self) -> &Option<IdContainer<SensorId, f32>> {
        &self.target
    }
}
