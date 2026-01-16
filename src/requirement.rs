use std::any::TypeId;

use crate::prelude::SensorValue;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Requirement<Id> {
    pub(crate) comparison: Comparison,
    pub(crate) id: Id,
}

impl Requirement<TypeId> {
    pub fn equal<T>(value: impl Into<SensorValue>) -> Self
    where
        T: ?Sized + 'static,
    {
        Self {
            id: TypeId::of::<T>(),
            comparison: Comparison::Equal(value.into()),
        }
    }
    pub fn not_equal<T>(value: impl Into<SensorValue>) -> Self
    where
        T: ?Sized + 'static,
    {
        Self {
            id: TypeId::of::<T>(),
            comparison: Comparison::NotEqual(value.into()),
        }
    }
}
