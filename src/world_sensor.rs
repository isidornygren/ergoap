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

#[cfg(feature = "target")]
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Hash)]
pub struct TargetValue {
    pub entity: Entity,
    pub is_close: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, PartialOrd)]
pub enum SensorValue {
    Bool(bool),
    #[cfg(feature = "target")]
    Target(Option<TargetValue>),
}

#[cfg(feature = "target")]
impl SensorValue {
    #[must_use]
    pub const fn has_target(&self) -> bool {
        match self {
            Self::Target(v) => v.is_some(),
            _ => false,
        }
    }

    #[must_use]
    pub const fn is_close(&self) -> bool {
        match self {
            Self::Target(Some(v)) => v.is_close,
            _ => false,
        }
    }

    #[must_use]
    pub const fn as_bool(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Target(Some(_)) => true,
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

#[queryable]
pub trait WorldSensor: Any {
    fn sensor_value(&self) -> SensorValue;
}

pub trait WorldSensorValue<T> {
    fn value(&self) -> T;
}

pub trait SensorComparison<T: Into<SensorValue>>: WorldSensorValue<T> + Any + Sized {
    #[must_use]
    fn equal(value: T) -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::Equal(value.into()))
    }

    #[must_use]
    fn not_equal(value: T) -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::NotEqual(value.into()))
    }
}
pub trait SensorComparisonOption<U>: SensorComparison<Option<U>>
where
    SensorValue: From<Option<U>>,
{
    #[must_use]
    fn is_some() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::IsSome)
    }
    #[must_use]
    fn is_none() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::IsNone)
    }
}

pub trait SensorComparisonBool: SensorComparison<bool>
where
    SensorValue: From<bool>,
{
    #[must_use]
    fn is_true() -> IdContainer<TypeId, Comparison> {
        IdContainer::new::<Self>(Comparison::Equal(SensorValue::Bool(true)))
    }
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

pub trait SensorEffect<T: Into<SensorValue>>: WorldSensorValue<T> + Any + Sized {
    fn set(value: T) -> IdContainer<TypeId, EffectValue> {
        IdContainer::new::<Self>(EffectValue::Set(value.into()))
    }
}

#[derive(Error, Debug)]
pub enum CollectSensorValuesError {
    #[error("component id not found for type id {type_id:?}:{entity:?}")]
    ComponentIdNotFound { type_id: TypeId, entity: Entity },
}

pub fn collect_sensor_values(
    mut commands: Commands,
    query: Query<(Entity, &dyn WorldSensor, Option<&SensorState>)>,
    components: &Components,
) -> Result {
    for (entity, entity_sensors, maybe_previous_state) in &query {
        let mut sensor_state = SensorState::with_capacity(entity_sensors.iter().count());

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
