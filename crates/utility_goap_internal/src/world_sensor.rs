use std::any::{Any, TypeId};

use bevy_ecs::{
    entity::Entity,
    error::Result,
    system::{Commands, Query},
    world::World,
};
use bevy_trait_query::queryable;
use thiserror::Error;

use crate::{Comparison, IdContainer, effect::EffectValue, sensor_state::SensorState};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd)]
pub enum SensorValue {
    Bool(bool),
    Integer(i32),
}

impl From<bool> for SensorValue {
    fn from(boolean: bool) -> SensorValue {
        SensorValue::Bool(boolean)
    }
}

impl From<i32> for SensorValue {
    fn from(value: i32) -> SensorValue {
        SensorValue::Integer(value)
    }
}

#[queryable]
pub trait WorldSensor {
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
    query: Query<(Entity, &dyn WorldSensor)>,
    world: &World,
) -> Result {
    for (entity, entity_sensors) in query.into_iter() {
        let mut sensor_state = SensorState::new();

        for sensor in entity_sensors {
            let value = sensor.sensor_value();
            let type_id = (*sensor).type_id();

            let id = world
                .components()
                .get_id(type_id)
                .ok_or(CollectSensorValuesError::ComponentIdNotFound { type_id, entity })?;

            sensor_state.insert(id, value);
        }

        commands.entity(entity).insert(sensor_state);
    }
    Ok(())
}
