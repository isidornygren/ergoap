use crate::SensorValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectValue {
    Set(SensorValue),
}
