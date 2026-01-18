use crate::SensorValue;

#[derive(Clone, Debug)]
pub enum Comparison {
    Equal(SensorValue),
    NotEqual(SensorValue),
}

impl Comparison {
    pub fn compare(&self, value: SensorValue) -> bool {
        match self {
            Self::Equal(v) => *v == value,
            Self::NotEqual(v) => *v != value,
        }
    }
}
