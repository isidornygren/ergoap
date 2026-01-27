//! Effects that actions have on the world state.
//!
//! This module defines how actions modify sensor values during planning.
//! Effects are applied to simulate the result of performing an action.

use crate::SensorValue;

/// An effect that an action has on a sensor value.
///
/// Effects are used during GOAP planning to simulate how performing an action
/// changes the world state. The planner applies effects to determine if an
/// action sequence achieves the desired goal.
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// // Create an effect that sets a sensor to true
/// let effect = MySensor::set(true);
/// ```
#[derive(Clone, Debug)]
pub enum EffectValue {
    /// Set the sensor value to a specific value.
    ///
    /// This replaces the current sensor value with the specified value.
    /// During planning, this effect is applied to the simulated world state.
    Set(SensorValue),
}
