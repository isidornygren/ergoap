use crate::SensorValue;

#[derive(Clone, Debug)]
pub enum EffectValue {
    Set(SensorValue),
}
