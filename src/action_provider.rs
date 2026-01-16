use std::any::TypeId;

use bevy_ecs::{
    component::{Component, ComponentId, Mutable, StorageType},
    lifecycle::HookContext,
    world::DeferredWorld,
};
use bevy_reflect::{PartialReflect, Reflect};
use bevy_trait_query::{RegisterExt, queryable};

use crate::{
    effects::{Effect, EffectValue},
    prelude::Requirement,
    sensor_state::SensorState,
};

#[queryable]
pub trait ActionProviderTrait {
    fn apply(&self, sensor_values: &mut SensorState);
    fn preconditions_met(&self, _sensor_values: &SensorState) -> bool;
    fn cost(&self) -> usize;
    fn component(&self) -> &dyn PartialReflect;
}

pub fn on_insert_action_provider_builder<C: Component + Reflect + Clone>(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(action_provider_builder) = world.get::<ActionProviderBuilder<C>>(entity) {
        let action_provider = action_provider_builder.build(&world);
        unsafe {
            world
                .as_unsafe_world_cell()
                .world_mut()
                .register_component_as::<dyn ActionProviderTrait, ActionProvider<C>>();
        }
        world
            .commands()
            .entity(entity)
            .insert(action_provider)
            .remove_by_id(component_id);
    }
}

#[derive(Clone, Default)]
pub struct ActionProviderBuilder<C: Component + Reflect> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<Requirement<TypeId>>,
    pub effects: Vec<Effect<TypeId>>,
}

impl<C: Component + Reflect> ActionProviderBuilder<C> {
    pub fn with_cost(mut self, cost: usize) -> Self {
        self.cost = cost;
        self
    }

    pub fn with_effect(mut self, effect: Effect<TypeId>) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn with_requirement(mut self, requirement: Requirement<TypeId>) -> Self {
        self.requirements.push(requirement);
        self
    }
}

impl<C: Component + Reflect + Clone> Component for ActionProviderBuilder<C> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Mutable;

    fn on_insert() -> Option<bevy_ecs::lifecycle::ComponentHook> {
        Some(on_insert_action_provider_builder::<C>)
    }
}

impl<C: Component + Reflect + Clone> ActionProviderBuilder<C> {
    pub(crate) fn build(&self, world: &DeferredWorld) -> ActionProvider<C> {
        ActionProvider {
            cost: self.cost,
            action: self.action.clone(),
            requirements: self
                .requirements
                .iter()
                .map(|Requirement { id, comparison }| Requirement {
                    id: world
                        .components()
                        .get_id(*id)
                        .expect("Could not get requirement id"),
                    comparison: comparison.clone(),
                })
                .collect(),
            effects: self
                .effects
                .iter()
                .map(|Effect { id, value }| Effect {
                    id: world
                        .components()
                        .get_id(*id)
                        .expect("Could not get component id"),
                    value: value.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Component, Clone)]
#[require(SensorState)]
pub struct ActionProvider<C: Component + Reflect> {
    pub action: C,
    pub cost: usize,
    pub requirements: Vec<Requirement<ComponentId>>,
    pub effects: Vec<Effect<ComponentId>>,
}

impl<C: Component + Reflect> ActionProvider<C> {
    pub fn new(action: C) -> ActionProviderBuilder<C> {
        ActionProviderBuilder {
            action,
            cost: 1,
            requirements: vec![],
            effects: vec![],
        }
    }
}

impl<C: Component + Reflect + Default> Default for ActionProvider<C> {
    fn default() -> Self {
        Self {
            action: C::default(),
            cost: 1,
            requirements: vec![],
            effects: vec![],
        }
    }
}

impl<C: Component + Reflect> ActionProviderTrait for ActionProvider<C> {
    fn apply(&self, sensor_values: &mut SensorState) {
        for Effect {
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
        self.requirements
            .iter()
            .all(|Requirement { id, comparison }| {
                sensor_values
                    .get(id)
                    .map_or(false, |v| comparison.compare(*v))
            })
    }

    fn cost(&self) -> usize {
        self.cost
    }
    fn component(&self) -> &dyn PartialReflect {
        &self.action
    }
}
