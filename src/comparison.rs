//! Comparison operators for sensor values in preconditions.
//!
//! This module defines comparison operations used in action preconditions and goal requirements.
//! Comparisons are evaluated against sensor values to determine if conditions are met.

use crate::SensorValue;

/// Comparison operators for evaluating sensor values.
///
/// This enum represents different ways to compare sensor values in action preconditions
/// and goal requirements. Comparisons return `true` or `false` based on the sensor value.
///
/// # Examples
///
/// ```ignore
/// use utility_goap::Comparison;
/// use utility_goap::SensorValue;
///
/// let comparison = Comparison::Equal(SensorValue::Bool(true));
/// assert!(comparison.compare(&SensorValue::Bool(true)));
/// assert!(!comparison.compare(&SensorValue::Bool(false)));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Comparison {
    /// Check if the sensor value equals the given value
    Equal(SensorValue),
    /// Check if the sensor value does not equal the given value
    NotEqual(SensorValue),
    /// Check if the sensor value is greater than the given value
    GreaterThan(SensorValue),
    /// Check if the sensor value is less than the given value
    LessThan(SensorValue),
    /// Check if an optional sensor value is `Some` (with `target` feature)
    IsSome,
    /// Check if an optional sensor value is `None` (with `target` feature)
    IsNone,
}

impl Comparison {
    /// Evaluate this comparison against a sensor value.
    ///
    /// # Arguments
    ///
    /// * `value` - The sensor value to compare against
    ///
    /// # Returns
    ///
    /// `true` if the comparison is satisfied, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let comparison = Comparison::GreaterThan(SensorValue::Bool(false));
    /// assert!(comparison.compare(&SensorValue::Bool(true)));
    /// ```
    #[must_use]
    pub fn compare(&self, value: &SensorValue) -> bool {
        match self {
            Self::Equal(v) => *v == *value,
            Self::NotEqual(v) => *v != *value,
            Self::GreaterThan(v) => *v < *value,
            Self::LessThan(v) => *v > *value,
            Self::IsSome => matches!(value, SensorValue::Target(Some(_))),
            Self::IsNone => matches!(value, SensorValue::Target(None)),
        }
    }
}
