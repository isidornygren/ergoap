#[cfg(feature = "target")]
use std::any::Any;
use std::any::TypeId;

use bevy_ecs::{
    component::{Component, ComponentId, Mutable, StorageType},
    lifecycle::HookContext,
    reflect::AppTypeRegistry,
    world::{DeferredWorld, World},
};
use bevy_reflect::{FromReflect, GetTypeRegistration, PartialReflect, Reflect, Typed};
use bevy_trait_query::queryable;

use crate::{
    Comparison, IdContainer, RegisterComponentAs, current_action::CurrentAction,
    effect::EffectValue, id_container::ComponentNotFound, sensor_state::SensorState,
};

#[queryable]
pub trait ActionProviderTrait: Send + Sync {
    fn apply(&self, sensor_values: &mut SensorState);
    fn preconditions_met(&self, _sensor_values: &SensorState) -> bool;
    fn cost(&self) -> usize;
    fn component(&self) -> &dyn PartialReflect;
    fn clone_box(&self) -> Box<dyn ActionProviderTrait>;
    #[cfg(feature = "target")]
    fn target(&self) -> &Option<ComponentId>;
}

pub fn on_insert_action_provider_builder<C: Clone + Typed + FromReflect + GetTypeRegistration>(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(action_provider_builder) = world.get::<ActionProviderBuilder<C>>(entity) {
        let action_provider = action_provider_builder
            .build(&world)
            .expect("Could not build action provider");
        world
            .resource_mut::<AppTypeRegistry>()
            .write()
            .register::<CurrentAction<C>>();
        world
            .commands()
            .entity(entity)
            .insert(action_provider)
            .remove_by_id(component_id);
    }
}

#[derive(Clone)]
pub struct ActionProviderBuilder<C> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<IdContainer<TypeId, Comparison>>,
    pub effects: Vec<IdContainer<TypeId, EffectValue>>,
    #[cfg(feature = "target")]
    pub target: Option<TypeId>,
}

impl<C> ActionProviderBuilder<C> {
    pub fn with_cost(mut self, cost: usize) -> Self {
        self.cost = cost;
        self
    }

    pub fn with_effect(mut self, effect: IdContainer<TypeId, EffectValue>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn with_requirement(mut self, requirement: IdContainer<TypeId, Comparison>) -> Self {
        self.requirements.push(requirement);
        self
    }

    #[cfg(feature = "target")]
    pub fn with_target<T: Any>(mut self) -> Self {
        self.target = Some(TypeId::of::<T>());
        self
    }
}

impl<C: Clone + Typed + FromReflect + GetTypeRegistration> Component for ActionProviderBuilder<C> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Mutable;

    fn on_insert() -> Option<bevy_ecs::lifecycle::ComponentHook> {
        Some(on_insert_action_provider_builder::<C>)
    }
}

impl<C: Clone + Typed + FromReflect + GetTypeRegistration> ActionProviderBuilder<C> {
    pub(crate) fn build(
        &self,
        world: &World,
    ) -> Result<ActionProvider<CurrentAction<C>>, ComponentNotFound> {
        Ok(ActionProvider {
            cost: self.cost,
            action: CurrentAction {
                action: self.action.clone(),
            },
            requirements: self
                .requirements
                .iter()
                .map(|requirement| requirement.build(world))
                .collect::<Result<Vec<_>, _>>()?,
            effects: self
                .effects
                .iter()
                .map(|effect| effect.build(world))
                .collect::<Result<Vec<_>, _>>()?,
            #[cfg(feature = "target")]
            target: match self.target {
                Some(target) => Some(
                    world
                        .components()
                        .get_id(target)
                        .ok_or(ComponentNotFound(target))?,
                ),
                None => None,
            },
        })
    }
}

#[derive(Component, Clone)]
#[require(SensorState)]
pub struct ActionProvider<C: Reflect> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<IdContainer<ComponentId, Comparison>>,
    pub effects: Vec<IdContainer<ComponentId, EffectValue>>,
    #[cfg(feature = "target")]
    pub target: Option<ComponentId>,
}

impl<C: Reflect + Clone + RegisterComponentAs> ActionProvider<C> {
    pub fn new(action: C) -> ActionProviderBuilder<C> {
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

impl<C: Reflect + Clone> ActionProviderTrait for ActionProvider<C> {
    fn apply(&self, sensor_values: &mut SensorState) {
        for IdContainer {
            id,
            value: effect_value,
        } in self.effects.iter()
        {
            match effect_value {
                EffectValue::Set(value) => sensor_values.insert(*id, *value),
            }
        }
    }

    fn preconditions_met(&self, sensor_values: &SensorState) -> bool {
        #[cfg(feature = "target")]
        if let Some(target) = self.target {
            if sensor_values.get(&target).is_none_or(|v| !v.has_target()) {
                return false;
            }
        }
        self.requirements.iter().all(|IdContainer { id, value }| {
            sensor_values.get(id).map_or(false, |v| value.compare(v))
        })
    }

    fn cost(&self) -> usize {
        self.cost
    }
    fn component(&self) -> &dyn PartialReflect {
        &self.action
    }
    fn clone_box(&self) -> Box<dyn ActionProviderTrait> {
        Box::new(self.clone())
    }
    #[cfg(feature = "target")]
    fn target(&self) -> &Option<ComponentId> {
        &self.target
    }
}
