//! Sensor state storage and management.
//!
//! This module provides the [`SensorState`] component which stores the current values
//! of all sensors on an entity. The sensor state is used during planning to evaluate
//! action preconditions and goal satisfaction.

use std::collections::HashMap;

use bevy_ecs::component::{Component, ComponentId};

use crate::SensorValue;

/// Component storing the current sensor values for an entity.
///
/// This component is automatically populated by the sensor collection system and
/// used during GOAP planning to evaluate preconditions and goals. Each sensor
/// value is indexed by its component ID for efficient lookup.
///
/// # Hashing
///
/// The hash implementation ensures deterministic ordering by sorting keys before hashing.
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// // Sensor state is typically managed automatically, but can be created manually:
/// let mut state = SensorState::new();
/// state.insert(component_id, true.into());
/// ```
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
    /// Create a new empty sensor state.
    ///
    /// This is typically not needed as [`SensorState`] implements [`Default`].
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Create a sensor state from an iterator of component ID and value pairs.
    ///
    /// # Type Parameters
    ///
    /// * `V` - An iterator of `(ComponentId, I)` pairs
    /// * `I` - Any type that can be converted into [`SensorValue`]
    ///
    /// # Example
    ///
    /// ```ignore
    /// let state = SensorState::from_vec(vec![
    ///     (component_id_1, true),
    ///     (component_id_2, false),
    /// ]);
    /// ```
    #[must_use]
    pub fn from_vec<V: IntoIterator<Item = (ComponentId, I)>, I: Into<SensorValue>>(
        values: V,
    ) -> Self {
        Self(
            values
                .into_iter()
                .map(|(comp_id, value)| (comp_id, value.into()))
                .collect(),
        )
    }

    /// Get the sensor value for a specific component ID.
    ///
    /// # Arguments
    ///
    /// * `component_id` - The component ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&SensorValue)` - The sensor value if it exists
    /// * `None` - If no value exists for this component ID
    #[must_use]
    pub fn get(&self, component_id: &ComponentId) -> Option<&SensorValue> {
        self.0.get(component_id)
    }

    /// Insert or update a sensor value.
    ///
    /// # Arguments
    ///
    /// * `component_id` - The component ID to associate with the value
    /// * `value` - The sensor value (automatically converted to [`SensorValue`])
    pub fn insert<T: Into<SensorValue>>(&mut self, component_id: ComponentId, value: T) {
        self.0.insert(component_id, value.into());
    }

    /// Iterate over all component ID and sensor value pairs.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to `(ComponentId, SensorValue)` pairs
    pub fn iter(&self) -> impl Iterator<Item = (&ComponentId, &SensorValue)> {
        self.0.iter()
    }
}
