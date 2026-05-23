use std::any::TypeId;

use crate::SensorValue;
use bevy_ecs::component::{Component, ComponentId};
use bitvec::prelude::*;
use fxhash::FxHashMap;

#[derive(Clone, Copy, Debug)]
pub struct SensorId(pub(crate) usize);

#[derive(Component, Debug, Default, Clone)]
pub struct SensorState {
    pub(crate) values: Vec<SensorValue>,
    pub(crate) type_id_map: FxHashMap<TypeId, SensorId>,
}

impl SensorState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: vec![],
            type_id_map: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            type_id_map: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn from_vec<V: IntoIterator<Item = (ComponentId, I)> + Clone, I: Into<SensorValue>>(
        values: V,
    ) -> Self {
        Self {
            values: values.into_iter().map(|(_, value)| value.into()).collect(),
            type_id_map: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn bit_vec(&self) -> BitVec {
        self.values
            .iter()
            .map(SensorValue::as_bool)
            .collect::<BitVec>()
    }

    #[must_use]
    pub fn get(&self, id: &SensorId) -> Option<&SensorValue> {
        self.values.get(id.0)
    }

    pub fn push<T: Into<SensorValue>>(&mut self, type_id: TypeId, value: T) -> SensorId {
        self.values.push(value.into());
        let sensor_id = SensorId(self.values.len() - 1);
        self.type_id_map.insert(type_id, sensor_id);
        sensor_id
    }

    pub fn push_empty(&mut self, type_id: TypeId) -> SensorId {
        self.values.push(SensorValue::None);
        let sensor_id = SensorId(self.values.len() - 1);
        self.type_id_map.insert(type_id, sensor_id);
        sensor_id
    }

    pub fn insert<T: Into<SensorValue>>(&mut self, id: SensorId, value: T) {
        self.values[id.0] = value.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_state_into_bitvec_u8() {
        let mut sensor_vec = vec![];
        for index in 0..8 {
            sensor_vec.push((ComponentId::new(index), SensorValue::Bool(index % 2 == 0)));
        }

        let sensor_state = SensorState::from_vec(sensor_vec);

        let bit_vec = sensor_state.bit_vec();

        assert_eq!(bit_vec.as_raw_slice(), [0b0101_0101]);
    }

    #[test]
    fn sensor_state_into_bitvec_u64() {
        let mut sensor_vec = vec![];
        for index in 0..64 {
            sensor_vec.push((ComponentId::new(index), SensorValue::Bool(index % 2 == 0)));
        }

        let sensor_state = SensorState::from_vec(sensor_vec);

        let bit_vec = sensor_state.bit_vec();

        assert_eq!(
            bit_vec.as_raw_slice(),
            [0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101]
        );
    }

    #[test]
    fn sensor_state_into_bitvec_2x_u64() {
        let mut sensor_vec = vec![];
        for index in 0..128 {
            sensor_vec.push((ComponentId::new(index), SensorValue::Bool(index % 2 == 0)));
        }

        let sensor_state = SensorState::from_vec(sensor_vec);

        let bit_vec = sensor_state.bit_vec();

        assert_eq!(
            bit_vec.as_raw_slice(),
            [
                0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101,
                0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101
            ]
        );
    }
}
