use std::any::{Any, TypeId};

use bevy_ecs::{component::ComponentId, world::World};
use thiserror::Error;

#[derive(Clone, Debug)]
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

impl<Value: Clone> IdContainer<TypeId, Value> {
    pub fn build(
        &self,
        world: &World,
    ) -> Result<IdContainer<ComponentId, Value>, ComponentNotFound> {
        let id = world
            .components()
            .get_id(self.id)
            .ok_or(ComponentNotFound(self.id))?;

        Ok(IdContainer {
            id,
            value: self.value.clone(),
        })
    }
}
