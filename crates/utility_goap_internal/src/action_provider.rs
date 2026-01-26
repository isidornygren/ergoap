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

#[cfg(feature = "target")]
use crate::target::TargetConfig;
use crate::{
    Comparison, IdContainer, current_action::CurrentAction, effect::EffectValue,
    id_container::ComponentNotFound, sensor_state::SensorState,
};

#[queryable]
pub trait ActionProviderTrait: Send + Sync {
    fn apply(&self, sensor_values: &mut SensorState);
    fn preconditions_met(&self, _sensor_values: &SensorState) -> bool;
    fn cost(&self) -> usize;
    fn insert_current_action(&self, entity_world: &mut EntityWorldMut);
    fn clone_box(&self) -> Box<dyn ActionProviderTrait>;
    #[cfg(feature = "target")]
    fn target(&self) -> &Option<IdContainer<ComponentId, TargetConfig>>;
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
                let action_provider = action_provider_builder.build(entity_world_mut.world())?;
                entity_world_mut.insert(action_provider);
            }
            Ok(())
        });
}

pub struct ActionProviderBuilder<C> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<IdContainer<TypeId, Comparison>>,
    pub effects: Vec<IdContainer<TypeId, EffectValue>>,
    #[cfg(feature = "target")]
    pub target: Option<IdContainer<TypeId, TargetConfig>>,
}

impl<C> ActionProviderBuilder<C> {
    #[must_use]
    pub const fn with_cost(mut self, cost: usize) -> Self {
        self.cost = cost;
        self
    }

    #[must_use]
    pub fn with_effect(mut self, effect: IdContainer<TypeId, EffectValue>) -> Self {
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
    pub const fn with_target<T: Any>(mut self, target_config: TargetConfig) -> Self {
        self.target = Some(IdContainer {
            value: target_config,
            id: TypeId::of::<T>(),
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
}

impl<C> ActionProviderBuilder<C> {
    pub(crate) fn build(self, world: &World) -> Result<ActionProvider<C>, ComponentNotFound> {
        Ok(ActionProvider {
            cost: self.cost,
            action: self.action,
            requirements: self
                .requirements
                .into_iter()
                .map(|requirement| requirement.build(world))
                .collect::<Result<Vec<_>, _>>()?,
            effects: self
                .effects
                .into_iter()
                .map(|effect| effect.build(world))
                .collect::<Result<Vec<_>, _>>()?,
            #[cfg(feature = "target")]
            target: match &self.target {
                Some(target) => Some(target.build(world)?),
                None => None,
            },
        })
    }
}

#[derive(Component, Clone)]
#[require(SensorState)]
pub struct ActionProvider<C> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<IdContainer<ComponentId, Comparison>>,
    pub effects: Vec<IdContainer<ComponentId, EffectValue>>,
    #[cfg(feature = "target")]
    pub target: Option<IdContainer<ComponentId, TargetConfig>>,
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
            && sensor_values
                .get(&target.id)
                .is_none_or(|v| !v.has_target())
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
    fn target(&self) -> &Option<IdContainer<ComponentId, TargetConfig>> {
        &self.target
    }
}
