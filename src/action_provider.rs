#[cfg(feature = "target")]
use std::any::Any;
use std::any::TypeId;

use bevy_asset::Asset;
use bevy_ecs::{
    component::{Component, ComponentId, Immutable, StorageType},
    error::Result,
    lifecycle::HookContext,
    world::{DeferredWorld, EntityWorldMut},
};
use bevy_reflect::TypePath;
use bitvec::vec::BitVec;

use crate::{
    Comparison, IdContainer, SensorValue,
    current_action::{CurrentActionRef, UpdateActionRef},
    id_container::{BuildSensorId, BuildSensorIdError},
    sensor_state::{SensorId, SensorState},
};
#[cfg(feature = "target")]
use crate::{TargetValue, WorldSensorValue};

trait ErasedActionComponent: Send + Sync {
    fn insert_into_entity_world(&self, entity_world: &mut EntityWorldMut);
    fn component_id(&self, entity_world: &mut EntityWorldMut) -> Option<ComponentId>;
    fn clone_box(&self) -> Box<dyn ErasedActionComponent>;
}

#[derive(Clone)]
struct TypedActionComponent<C>(C)
where
    C: Component + Clone + Send + Sync + 'static;

impl<C> ErasedActionComponent for TypedActionComponent<C>
where
    C: Component + Clone + Send + Sync + 'static,
{
    fn insert_into_entity_world(&self, entity_world: &mut EntityWorldMut) {
        entity_world.insert(self.0.clone());
    }

    fn component_id(&self, entity_world: &mut EntityWorldMut) -> Option<ComponentId> {
        entity_world.world_scope(|w| w.component_id::<C>())
    }

    fn clone_box(&self) -> Box<dyn ErasedActionComponent> {
        Box::new(self.clone())
    }
}

pub fn on_insert_action_provider_builder<C: Clone + Send + Sync + 'static + Component>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    println!("On insert action provider builder for entity {:?}", entity);
    world
        .commands()
        .entity(entity)
        .queue(|mut entity_world_mut: EntityWorldMut| -> Result {
            println!("Building action provider for entity");
            if let Some(action_provider_builder) =
                entity_world_mut.take::<ActionProviderBuilder<C>>()
            {
                let action_provider = action_provider_builder.build(&mut entity_world_mut)?;

                entity_world_mut.insert_if_new(ActionProviders(vec![]));

                if let Some(mut action_providers) = entity_world_mut.get_mut::<ActionProviders>() {
                    action_providers.0.push(action_provider);
                }
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

impl<C: Clone + Send + Sync + 'static + Component> Component for ActionProviderBuilder<C> {
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

impl<C: Component + Clone + Send + Sync + 'static> ActionProviderBuilder<C> {
    pub(crate) fn build(
        self,
        world: &mut EntityWorldMut,
    ) -> Result<ActionProvider, BuildSensorIdError> {
        Ok(ActionProvider {
            cost: self.cost,
            action: Box::new(TypedActionComponent(self.action)),
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
/// #[derive(Component, Clone)]
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
///
#[derive(Asset, TypePath)]
pub struct ActionProvider {
    /// The action that will spawn in the entity when the planner has chosen this ``ActionProvider``.
    action: Box<dyn ErasedActionComponent>,
    /// The cost of the action, the higher the cost, the more expensive the action.
    pub cost: usize,
    /// The state requirements for this action to be valid.
    /// If _all_ requirements are not met, the action cannot be selected.
    pub requirements: Vec<IdContainer<SensorId, Comparison>>, // SensorIds are per entity, not per world...... So using them in an asset would not work.
    /// The state effects of this action.
    pub effects: Vec<IdContainer<SensorId, SensorValue>>,
    #[cfg(feature = "target")]
    /// The target of the action, if the ``TargetValue`` does not have an entity it will not be able to be chosen.
    /// If the ``TargetValue``s ``is_close`` field is not ``true``, it will spawn a ``CurrentAction<GotoTarget>``.
    pub target: Option<IdContainer<SensorId, f32>>,
}

#[derive(Component)]
pub struct ActionProviders(pub Vec<ActionProvider>);

impl Clone for ActionProvider {
    fn clone(&self) -> Self {
        Self {
            action: self.action.clone_box(),
            cost: self.cost,
            requirements: self.requirements.clone(),
            effects: self.effects.clone(),
            #[cfg(feature = "target")]
            target: self.target,
        }
    }
}

impl ActionProvider {
    pub const fn new<C>(action: C) -> ActionProviderBuilder<C> {
        ActionProviderBuilder {
            action,
            cost: 1,
            requirements: vec![],
            effects: vec![],
            #[cfg(feature = "target")]
            target: None,
        }
    }

    pub fn from_action_with_cost<C>(action: C, cost: usize) -> Self
    where
        C: Component + Clone + Send + Sync + 'static,
    {
        Self {
            action: Box::new(TypedActionComponent(action)),
            cost,
            requirements: vec![],
            effects: vec![],
            #[cfg(feature = "target")]
            target: None,
        }
    }

    pub fn apply(&self, sensor_values: &mut SensorState) {
        for IdContainer { id, value } in &self.effects {
            sensor_values.insert(*id, *value);
        }
    }

    pub fn apply_to_bitvec(&self, bitvec: &mut BitVec) {
        for IdContainer { id, value } in &self.effects {
            bitvec.set(id.0, value.as_bool());
        }
    }

    #[must_use]
    pub fn preconditions_met(&self, sensor_values: &SensorState) -> bool {
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

    #[must_use]
    pub const fn cost(&self) -> usize {
        self.cost
    }

    pub fn add_to_entity_world(&self, entity_world: &mut EntityWorldMut) {
        if let Some(component_id) = self.action.component_id(entity_world) {
            entity_world.update_action_ref(component_id);
        } else {
            // This should probably not happen, but adding it here as a failsafe if a component is not registered.
            entity_world.remove::<CurrentActionRef>();
        }
        self.action.insert_into_entity_world(entity_world);
    }

    #[cfg(feature = "target")]
    #[must_use]
    pub const fn target(&self) -> &Option<IdContainer<SensorId, f32>> {
        &self.target
    }
}
