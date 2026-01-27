//! Automatic trait registration system for GOAP components.
//!
//! This module provides infrastructure for automatically registering component types
//! with Bevy's trait query system. It uses the `inventory` crate to collect registration
//! functions at compile time.

use bevy_ecs::world::World;

/// A wrapper type for trait registration functions.
///
/// This type is used with the `inventory` crate to collect all trait registration
/// functions that need to be called during plugin initialization.
pub struct AutomaticTraitRegistrations(pub fn(&mut World));

/// Register all collected trait types with the world.
///
/// This function is called during plugin initialization to register all components
/// that implement trait queries (like [`ActionProviderTrait`] and [`WorldSensor`]).
/// It iterates through all registration functions collected by the `inventory` crate.
///
/// [`ActionProviderTrait`]: crate::ActionProviderTrait
/// [`WorldSensor`]: crate::WorldSensor
pub fn register_trait_types(registry: &mut World) {
    for registration_fn in inventory::iter::<AutomaticTraitRegistrations> {
        registration_fn.0(registry);
    }
}

/// Trait for components that can register themselves for trait queries.
///
/// This trait is typically implemented automatically by derive macros and should not
/// be implemented manually. It provides a registration function that is called during
/// plugin initialization to set up trait queries.
pub trait RegisterComponentAs {
    /// Register this component type for trait queries.
    ///
    /// # Arguments
    ///
    /// * `world` - The Bevy world to register the component with
    fn __register_as(world: &mut World);
}

inventory::collect!(AutomaticTraitRegistrations);
