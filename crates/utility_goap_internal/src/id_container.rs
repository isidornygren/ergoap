use std::any::{Any, TypeId};

use bevy_ecs::{component::ComponentId, world::World};
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct IdContainer<Id, Value> {
    pub(crate) value: Value,
    pub(crate) id: Id,
}

impl<Value> IdContainer<TypeId, Value> {
    pub const fn new<T: Any>(value: Value) -> Self {
        Self {
            id: TypeId::of::<T>(),
            value,
        }
    }
}

#[derive(Error, Debug)]
#[error("Component with TypeId {0:?} not found in world")]
pub struct ComponentNotFound(pub TypeId);

pub trait BuildComponentId {
    fn build_id_container<Value: Clone>(
        &self,
        id_container: IdContainer<TypeId, Value>,
    ) -> Result<IdContainer<ComponentId, Value>, ComponentNotFound> {
        Ok(IdContainer {
            id: self.get_component_id(&id_container.id)?,
            value: id_container.value.clone(),
        })
    }

    fn get_component_id(&self, type_id: &TypeId) -> Result<ComponentId, ComponentNotFound>;
}

impl BuildComponentId for &World {
    fn get_component_id(&self, type_id: &TypeId) -> Result<ComponentId, ComponentNotFound> {
        self.components()
            .get_id(*type_id)
            .ok_or(ComponentNotFound(*type_id))
    }
}
