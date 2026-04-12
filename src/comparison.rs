use crate::SensorValue;

/// A comparison value used for sensor comparisons.
///
/// # Example
/// ```rust
/// # use ergoap::Comparison;
/// let comparison = Comparison::Equal(true.into());
/// assert!(comparison.compare(true));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Comparison {
    Equal(SensorValue),
    NotEqual(SensorValue),
    IsSome,
    IsNone,
}

impl Comparison {
    #[must_use]
    pub fn compare(&self, value: impl Into<SensorValue>) -> bool {
        match self {
            Self::Equal(v) => *v == value.into(),
            Self::NotEqual(v) => *v != value.into(),
            Self::IsSome => matches!(value.into(), SensorValue::Target(Some(_))),
            Self::IsNone => matches!(value.into(), SensorValue::Target(None)),
        }
    }
}
