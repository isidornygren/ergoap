use crate::SensorValue;

#[derive(Clone, Debug)]
pub enum Comparison {
    Equal(SensorValue),
    NotEqual(SensorValue),
    GreaterThan(SensorValue),
    LessThan(SensorValue),
}

impl Comparison {
    pub fn compare(&self, value: &SensorValue) -> bool {
        match self {
            Self::Equal(v) => *v == *value,
            Self::NotEqual(v) => *v != *value,
            Self::GreaterThan(v) => *v < *value,
            Self::LessThan(v) => *v > *value,
        }
    }
}
