use std::any::{Any, TypeId};

use bevy_ecs::world::EntityWorldMut;
use thiserror::Error;

use crate::{SensorState, sensor_state::SensorId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub enum BuildSensorIdError {
    #[error("Sensor state not present for entity")]
    SensorStateNotPresent,
    #[error("Component with TypeId {0:?} not found in world")]
    SensorNotFound(TypeId),
}

pub trait BuildSensorId {
    fn build_sensor_container<Value: Clone>(
        &mut self,
        id_container: IdContainer<TypeId, Value>,
    ) -> Result<IdContainer<SensorId, Value>, BuildSensorIdError> {
        Ok(IdContainer {
            id: self.get_sensor_id(&id_container.id)?,
            value: id_container.value.clone(),
        })
    }

    fn get_sensor_id(&mut self, type_id: &TypeId) -> Result<SensorId, BuildSensorIdError>;
}

impl BuildSensorId for EntityWorldMut<'_> {
    fn get_sensor_id(&mut self, type_id: &TypeId) -> Result<SensorId, BuildSensorIdError> {
        let mut sensor_state = self
            .get_mut::<SensorState>()
            .ok_or(BuildSensorIdError::SensorStateNotPresent)?;

        if let Some(sensor_id) = sensor_state.type_id_map.get(type_id) {
            return Ok(*sensor_id);
        }
        Ok(sensor_state.push_empty(*type_id))
    }
}
