use crate::SensorValue;

#[derive(Clone, Copy, Debug)]
pub enum Comparison {
    Equal(SensorValue),
    NotEqual(SensorValue),
    GreaterThan(SensorValue),
    LessThan(SensorValue),
    IsSome,
    IsNone,
}

impl Comparison {
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
