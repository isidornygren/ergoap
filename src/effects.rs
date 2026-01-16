use std::any::TypeId;

use crate::prelude::SensorValue;

#[derive(Clone)]
pub enum EffectValue {
    Set(SensorValue),
}

#[derive(Clone)]
pub struct Effect<Id> {
    pub(crate) value: EffectValue,
    pub(crate) id: Id,
}

impl Effect<TypeId> {
    pub fn set<T>(value: impl Into<SensorValue>) -> Self
    where
        T: ?Sized + 'static,
    {
        Self {
            value: EffectValue::Set(value.into()),
            id: TypeId::of::<T>(),
        }
    }
}
