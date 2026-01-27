//! World sensor system for observing and querying entity state.
//!
//! This module provides the core sensor infrastructure for GOAP planning. Sensors
//! observe the world state and provide values that are used to evaluate action
//! preconditions and goal satisfaction.

use std::any::{Any, TypeId};

use bevy_ecs::{
    component::Components,
    entity::Entity,
    error::Result,
    system::{Commands, Query},
};
use bevy_trait_query::queryable;
use thiserror::Error;

use crate::{Comparison, IdContainer, effect::EffectValue, sensor_state::SensorState};

/// A target value containing an entity reference and proximity information.
///
/// This type is used with the `target` feature to represent targets that actions
/// can move toward and interact with. The `is_close` flag indicates whether the
/// entity is within interaction range.
#[cfg(feature = "target")]
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Hash)]
pub struct TargetValue {
    /// The target entity
    pub entity: Entity,
    /// Whether the entity is close enough to interact with the target
    pub is_close: bool,
}

/// A sensor value that can be stored in [`SensorState`].
///
/// Sensor values represent discrete observations about the world that can be
/// compared in action preconditions and goal requirements. All sensor types
/// must be convertible to this enum.
///
/// # Example
///
/// ```ignore
/// use utility_goap::SensorValue;
///
/// let bool_value = SensorValue::Bool(true);
/// let target_value = SensorValue::Target(Some(target));
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, PartialOrd)]
pub enum SensorValue {
    /// A boolean sensor value
    Bool(bool),
    /// An optional target value (with `target` feature)
    #[cfg(feature = "target")]
    Target(Option<TargetValue>),
}

#[cfg(feature = "target")]
impl SensorValue {
    /// Check if this sensor value contains a target.
    ///
    /// Returns `true` only if this is a `Target` variant with `Some` value.
    #[must_use]
    pub const fn has_target(&self) -> bool {
        match self {
            Self::Target(v) => v.is_some(),
            _ => false,
        }
    }

    /// Check if the target is close enough for interaction.
    ///
    /// Returns `true` only if this is a `Target(Some(TargetValue))` where
    /// `is_close` is true.
    #[must_use]
    pub const fn is_close(&self) -> bool {
        match self {
            Self::Target(Some(v)) => v.is_close,
            _ => false,
        }
    }
}

impl From<bool> for SensorValue {
    fn from(boolean: bool) -> Self {
        Self::Bool(boolean)
    }
}

#[cfg(feature = "target")]
impl From<Option<TargetValue>> for SensorValue {
    fn from(value: Option<TargetValue>) -> Self {
        Self::Target(value)
    }
}

#[cfg(feature = "target")]
impl From<TargetValue> for SensorValue {
    fn from(value: TargetValue) -> Self {
        Self::Target(Some(value))
    }
}

/// Trait for components that observe the world state.
///
/// Implement this trait on components to make them observable by the GOAP planning
/// system. Sensors provide values that are collected into [`SensorState`] and used
/// to evaluate action preconditions and goals.
///
/// The trait is marked with `#[queryable]` to enable dynamic trait queries.
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// #[derive(Component)]
/// struct HasFood(bool);
///
/// impl WorldSensor for HasFood {
///     fn sensor_value(&self) -> SensorValue {
///         SensorValue::Bool(self.0)
///     }
/// }
///
/// impl WorldSensorValue<bool> for HasFood {
///     fn value(&self) -> bool {
///         self.0
///     }
/// }
/// ```
#[queryable]
pub trait WorldSensor: Any {
    /// Get the current sensor value.
    ///
    /// This method is called by the sensor collection system to read the
    /// current state and store it in [`SensorState`].
    fn sensor_value(&self) -> SensorValue;
}

/// Trait for sensors that provide typed values.
///
/// This trait is used as a bound for sensor comparison and effect traits.
/// It allows sensors to be associated with a specific value type.
///
/// # Type Parameters
///
/// * `T` - The underlying value type of the sensor
pub trait WorldSensorValue<T> {
    /// Get the typed value from this sensor.
    fn value(&self) -> T;
}

/// Trait providing comparison methods for sensor values.
///
/// This trait is automatically implemented for sensors that implement
/// [`WorldSensorValue<T>`] where `T` can be converted to [`SensorValue`].
/// It provides convenient methods for creating comparison requirements.
///
/// # Type Parameters
///
/// * `T` - The value type that can be compared and converted to [`SensorValue`]
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// // Create requirements using comparison methods
/// let req1 = MySensor::equal(true);
/// let req2 = MySensor::not_equal(false);
/// let req3 = MySensor::greater_than(5);
/// ```
pub trait SensorComparison<T: Into<SensorValue>>: WorldSensorValue<T> + Any + Sized {
    /// Create a comparison requiring the sensor equals a specific value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to compare against
    #[must_use]
    fn equal(value: T) -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::Equal(value.into()))
    }

    /// Create a comparison requiring the sensor does not equal a specific value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to compare against
    #[must_use]
    fn not_equal(value: T) -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::NotEqual(value.into()))
    }

    /// Create a comparison requiring the sensor is greater than a specific value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to compare against
    #[must_use]
    fn greater_than(value: T) -> IdContainer<TypeId, Comparison>
    where
        T: PartialOrd,
    {
        IdContainer::new::<Self>(Comparison::GreaterThan(value.into()))
    }

    /// Create a comparison requiring the sensor is less than a specific value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to compare against
    #[must_use]
    fn less_than(value: T) -> IdContainer<TypeId, Comparison>
    where
        T: PartialOrd,
    {
        IdContainer::new::<Self>(Comparison::LessThan(value.into()))
    }
}
/// Trait providing comparison methods for optional sensor values.
///
/// This trait is automatically implemented for sensors with optional values
/// and provides methods for checking whether the value is present.
///
/// # Type Parameters
///
/// * `U` - The inner value type of the `Option`
///
/// # Example
///
/// ```ignore
/// // Create requirements for optional sensors
/// let has_target = TargetSensor::is_some();
/// let no_target = TargetSensor::is_none();
/// ```
pub trait SensorComparisonOption<U>: SensorComparison<Option<U>>
where
    SensorValue: From<Option<U>>,
{
    /// Create a comparison requiring the sensor value is `Some`.
    #[must_use]
    fn is_some() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::IsSome)
    }
    /// Create a comparison requiring the sensor value is `None`.
    #[must_use]
    fn is_none() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::IsNone)
    }
}

/// Trait providing comparison methods for boolean sensor values.
///
/// This trait is automatically implemented for sensors with boolean values
/// and provides convenient methods for creating true/false comparisons.
///
/// # Example
///
/// ```ignore
/// // Create boolean requirements
/// let must_be_true = MySensor::is_true();
/// let must_be_false = MySensor::is_false();
/// ```
pub trait SensorComparisonBool: SensorComparison<bool>
where
    SensorValue: From<bool>,
{
    /// Create a comparison requiring the sensor value is `true`.
    #[must_use]
    fn is_true() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::Equal(SensorValue::Bool(true)))
    }
    /// Create a comparison requiring the sensor value is `false`.
    #[must_use]
    fn is_false() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::Equal(SensorValue::Bool(false)))
    }
}

impl<T, U> SensorComparisonOption<U> for T
where
    T: SensorComparison<Option<U>>,
    SensorValue: From<Option<U>>,
{
}

impl<T> SensorComparisonBool for T where T: SensorComparison<bool> {}

/// Trait providing effect methods for sensor values.
///
/// This trait is automatically implemented for sensors and provides methods
/// for creating effects that modify sensor values during planning simulation.
///
/// # Type Parameters
///
/// * `T` - The value type that can be set and converted to [`SensorValue`]
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// // Create an effect that sets a sensor value
/// let effect = MySensor::set(true);
///
/// // Use in an action
/// let action = ActionProvider::new(MyAction)
///     .with_effect(MySensor::set(true));
/// ```
pub trait SensorEffect<T: Into<SensorValue>>: WorldSensorValue<T> + Any + Sized {
    /// Create an effect that sets the sensor to a specific value.
    ///
    /// This is used in action effects to specify how the action changes
    /// the world state during planning.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to set the sensor to
    fn set(value: T) -> IdContainer<TypeId, EffectValue> {
        IdContainer::new::<Self>(EffectValue::Set(value.into()))
    }
}

/// Errors that can occur during sensor value collection.
#[derive(Error, Debug)]
pub enum CollectSensorValuesError {
    /// A component ID could not be found for a sensor's type ID
    #[error("component id not found for type id {type_id:?}:{entity:?}")]
    ComponentIdNotFound {
        /// The type ID that couldn't be resolved
        type_id: TypeId,
        /// The entity being processed
        entity: Entity
    },
}

/// System that collects sensor values from world components.
///
/// This system runs in the [`SensorUpdate`] schedule and queries all entities
/// with [`WorldSensor`] components. It collects their values into [`SensorState`]
/// components, which are then used for planning.
///
/// # Behavior
///
/// - Queries all entities with sensor components
/// - Collects sensor values from each [`WorldSensor`] implementation
/// - Only updates [`SensorState`] if values have changed (for change detection)
/// - Resolves type IDs to component IDs for efficient storage
///
/// # Errors
///
/// Returns [`CollectSensorValuesError::ComponentIdNotFound`] if a sensor's
/// type cannot be resolved to a component ID.
///
/// [`SensorUpdate`]: crate::SensorUpdate
pub fn collect_sensor_values(
    mut commands: Commands,
    query: Query<(Entity, &dyn WorldSensor, Option<&SensorState>)>,
    components: &Components,
) -> Result {
    for (entity, entity_sensors, maybe_previous_state) in &query {
        let mut sensor_state = SensorState::new();

        for sensor in entity_sensors {
            let value = sensor.sensor_value();
            let type_id = (*sensor).type_id();

            let id = components
                .get_id(type_id)
                .ok_or(CollectSensorValuesError::ComponentIdNotFound { type_id, entity })?;

            sensor_state.insert(id, value);
        }

        if maybe_previous_state.is_none_or(|previous_state| *previous_state != sensor_state) {
            commands.entity(entity).insert(sensor_state);
        }
    }
    Ok(())
}
