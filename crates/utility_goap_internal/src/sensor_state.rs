use std::collections::HashMap;

use bevy_ecs::component::{Component, ComponentId};

use crate::SensorValue;

#[derive(Component, Debug, Default, PartialEq, Eq, Clone)]
pub struct SensorState(pub(crate) HashMap<ComponentId, SensorValue>);

impl std::hash::Hash for SensorState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<_> = self.0.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.0.get(key).hash(state);
        }
    }
}

impl SensorState {
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub fn from_vec<V, I>(values: V) -> Self
    where
        V: IntoIterator<Item = (ComponentId, I)>,
        I: Into<SensorValue>,
    {
        Self(
            values
                .into_iter()
                .map(|(comp_id, value)| (comp_id, value.into()))
                .collect(),
        )
    }

    #[must_use]
    pub fn get(&self, component_id: &ComponentId) -> Option<&SensorValue> {
        self.0.get(component_id)
    }

    pub fn insert<T>(&mut self, component_id: ComponentId, value: T)
    where
        T: Into<SensorValue>,
    {
        self.0.insert(component_id, value.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ComponentId, &SensorValue)> {
        self.0.iter()
    }
}
