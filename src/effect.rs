use crate::SensorValue;

#[derive(Clone, Debug, PartialEq)]
pub enum EffectValue {
    Set(SensorValue),
}
