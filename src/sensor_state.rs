use std::hash::BuildHasherDefault;

use crate::SensorValue;
use bevy_ecs::component::{Component, ComponentId};
use bitvec::prelude::*;
use fxhash::FxHashMap;

#[derive(Component, Debug, Default, PartialEq, Eq, Clone)]
pub struct SensorState {
    pub(crate) values: FxHashMap<usize, SensorValue>,
    pub(crate) keys: Vec<usize>,
}

impl SensorState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: FxHashMap::default(),
            keys: vec![],
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: FxHashMap::with_capacity_and_hasher(capacity, BuildHasherDefault::default()),
            keys: Vec::with_capacity(capacity),
        }
    }

    #[must_use]
    pub fn from_vec<V: IntoIterator<Item = (ComponentId, I)> + Clone, I: Into<SensorValue>>(
        values: V,
    ) -> Self {
        let mut keys: Vec<usize> = values
            .clone()
            .into_iter()
            .map(|(comp_id, _)| comp_id.index())
            .collect();
        keys.sort_unstable();
        Self {
            keys,
            values: values
                .into_iter()
                .map(|(comp_id, value)| (comp_id.index(), value.into()))
                .collect(),
        }
    }

    #[must_use]
    pub fn bit_vec(&self) -> BitVec {
        let mut bit_vec = bitvec![0u64; self.keys.len()];

        for (index, key) in self.keys.iter().enumerate() {
            let value = self.values.get(key);
            bit_vec.set(
                index,
                value.is_some_and(super::world_sensor::SensorValue::as_bool),
            );
        }

        bit_vec
    }

    #[must_use]
    pub fn get(&self, component_id: &ComponentId) -> Option<&SensorValue> {
        self.values.get(&component_id.index())
    }

    pub fn insert<T: Into<SensorValue>>(&mut self, component_id: ComponentId, value: T) {
        let index = component_id.index();
        if !self.values.contains_key(&index) {
            self.keys.push(index);
            self.keys.sort_unstable();
        }
        self.values.insert(index, value.into());
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
