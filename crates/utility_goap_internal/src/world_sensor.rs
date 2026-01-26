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
#[derive(Debug, PartialEq, Clone, Copy, PartialOrd, Hash)]
pub struct TargetValue {
    pub entity: Entity,
    pub is_close: bool,
}

#[derive(Debug, PartialEq, Clone, Hash, Copy, PartialOrd)]
pub enum SensorValue {
    Bool(bool),
    #[cfg(feature = "target")]
    Target(Option<TargetValue>),
}

#[cfg(feature = "target")]
impl SensorValue {
    pub fn has_target(&self) -> bool {
        match self {
            Self::Target(v) => v.is_some(),
            _ => false,
        }
    }

    pub fn is_close(&self) -> bool {
        match self {
            Self::Target(Some(v)) => v.is_close,
            _ => false,
        }
    }
}

impl From<bool> for SensorValue {
    fn from(boolean: bool) -> SensorValue {
        SensorValue::Bool(boolean)
    }
}

#[cfg(feature = "target")]
impl From<Option<TargetValue>> for SensorValue {
    fn from(value: Option<TargetValue>) -> SensorValue {
        SensorValue::Target(value)
    }
}

#[cfg(feature = "target")]
impl From<TargetValue> for SensorValue {
    fn from(value: TargetValue) -> SensorValue {
        SensorValue::Target(Some(value))
    }
}

#[queryable]
pub trait WorldSensor: Any {
    fn sensor_value(&self) -> SensorValue;
}

pub trait WorldSensorValue<T> {
    fn value(&self) -> T;
}

pub trait SensorComparison<T>: WorldSensorValue<T> + Any {
    fn equal(value: T) -> IdContainer<TypeId, Comparison>
    where
        T: Into<SensorValue>,
        Self: Sized,
    {
        IdContainer::new::<Self>(Comparison::Equal(value.into()))
    }

    fn not_equal(value: T) -> IdContainer<TypeId, Comparison>
    where
        T: Into<SensorValue>,
        Self: Sized,
    {
        IdContainer::new::<Self>(Comparison::NotEqual(value.into()))
    }

    fn greater_than(value: T) -> IdContainer<TypeId, Comparison>
    where
        T: Into<SensorValue> + PartialOrd,
        Self: Sized,
    {
        IdContainer::new::<Self>(Comparison::GreaterThan(value.into()))
    }

    fn less_than(value: T) -> IdContainer<TypeId, Comparison>
    where
        T: Into<SensorValue> + PartialOrd,
        Self: Sized,
    {
        IdContainer::new::<Self>(Comparison::LessThan(value.into()))
    }
}

pub trait SensorEffect<T>: WorldSensorValue<T> + Any {
    fn set(value: T) -> IdContainer<TypeId, EffectValue>
    where
        T: Into<SensorValue>,
        Self: Sized,
    {
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
