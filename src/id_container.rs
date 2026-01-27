//! ID container utilities for mapping between type IDs and component IDs.
//!
//! This module provides types for storing values associated with component identifiers.
//! It handles the conversion from compile-time [`TypeId`] to runtime [`ComponentId`] used by Bevy.

use std::any::{Any, TypeId};

use bevy_ecs::{component::ComponentId, world::World};
use thiserror::Error;

/// A container that pairs a value with an identifier (either [`TypeId`] or [`ComponentId`]).
///
/// This type is used throughout the GOAP system to associate values like comparisons
/// or effects with specific component types. It supports conversion from compile-time
/// [`TypeId`] to runtime [`ComponentId`].
///
/// # Type Parameters
///
/// * `Id` - The identifier type ([`TypeId`] or [`ComponentId`])
/// * `Value` - The value type stored with the identifier
#[derive(Clone, Copy, Debug)]
pub struct IdContainer<Id, Value> {
    pub(crate) value: Value,
    pub(crate) id: Id,
}

impl<Value> IdContainer<TypeId, Value> {
    /// Create a new ID container with a type-based identifier.
    ///
    /// This is typically used when constructing action requirements or effects,
    /// where the component type is known at compile time.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use as the identifier
    ///
    /// # Example
    ///
    /// ```ignore
    /// use utility_goap::IdContainer;
    /// use utility_goap::Comparison;
    ///
    /// let container = IdContainer::new::<MySensor>(Comparison::Equal(true.into()));
    /// ```
    pub const fn new<T: Any>(value: Value) -> Self {
        Self {
            id: TypeId::of::<T>(),
            value,
        }
    }
}

/// Error returned when attempting to resolve a [`TypeId`] to a [`ComponentId`] fails.
///
/// This typically happens when a component type hasn't been registered with the world.
#[derive(Error, Debug)]
#[error("Component with TypeId {0:?} not found in world")]
pub struct ComponentNotFound(pub TypeId);

/// Trait for types that can convert type IDs to component IDs.
///
/// This trait is implemented for Bevy's [`World`] and provides the infrastructure
/// for resolving compile-time type information to runtime component IDs.
pub trait BuildComponentId {
    /// Convert an ID container from [`TypeId`] to [`ComponentId`].
    ///
    /// # Errors
    ///
    /// Returns [`ComponentNotFound`] if the type ID hasn't been registered as a component.
    fn build_id_container<Value: Clone>(
        &self,
        id_container: IdContainer<TypeId, Value>,
    ) -> Result<IdContainer<ComponentId, Value>, ComponentNotFound> {
        Ok(IdContainer {
            id: self.get_component_id(&id_container.id)?,
            value: id_container.value.clone(),
        })
    }

    /// Get the component ID for a given type ID.
    ///
    /// # Errors
    ///
    /// Returns [`ComponentNotFound`] if the type ID hasn't been registered as a component.
    fn get_component_id(&self, type_id: &TypeId) -> Result<ComponentId, ComponentNotFound>;
}

impl BuildComponentId for &World {
    /// Look up the component ID for a type ID in this world.
    ///
    /// # Errors
    ///
    /// Returns [`ComponentNotFound`] if no component with the given type ID exists in the world.
    fn get_component_id(&self, type_id: &TypeId) -> Result<ComponentId, ComponentNotFound> {
        self.components()
            .get_id(*type_id)
            .ok_or(ComponentNotFound(*type_id))
    }
}
