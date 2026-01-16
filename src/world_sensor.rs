use std::any::{Any, TypeId};

use bevy_ecs::{
    entity::Entity,
    error::Result,
    system::{Commands, Query},
    world::World,
};
use bevy_trait_query::queryable;
use thiserror::Error;

use crate::sensor_state::SensorState;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SensorValue {
    Bool(bool),
}

impl From<bool> for SensorValue {
    fn from(boolean: bool) -> SensorValue {
        SensorValue::Bool(boolean)
    }
}

#[queryable]
pub trait WorldSensor: Any {
    fn sensor_value(&self) -> SensorValue;
}

#[derive(Error, Debug)]
pub enum CollectSensorValuesError {
    #[error("component id not found for type id {type_id:?}:{entity:?}")]
    ComponentIdNotFound { type_id: TypeId, entity: Entity },
}

pub fn collect_sensor_values(
    mut commands: Commands,
    query: Query<(Entity, &dyn WorldSensor)>,
    world: &World,
) -> Result {
    for (entity, entity_sensors) in query.into_iter() {
        let mut sensor_state = SensorState::new();

        for sensor in entity_sensors {
            let value = sensor.sensor_value();
            let type_id = (*sensor).type_id();

            let id = world
                .components()
                .get_id(type_id)
                .ok_or(CollectSensorValuesError::ComponentIdNotFound { type_id, entity })?;

            sensor_state.insert(id, value);
        }

        commands.entity(entity).insert(sensor_state);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy_app::{App, Update};
    use bevy_ecs::{component::Component, system::RunSystemOnce};
    use bevy_trait_query::RegisterExt;
    use std::collections::HashMap;

    use super::*;

    #[derive(Component)]
    struct TestSensor {
        pub active: bool,
    }

    impl WorldSensor for TestSensor {
        fn sensor_value(&self) -> SensorValue {
            SensorValue::Bool(self.active)
        }
    }

    #[test]
    fn collects_sensor_values() {
        let mut app = App::new();

        app.register_component_as::<dyn WorldSensor, TestSensor>()
            .add_systems(Update, collect_sensor_values);

        app.world_mut().spawn(TestSensor { active: false });

        app.update();

        let test_sensor_type_id = app
            .world()
            .components()
            .get_id(TypeId::of::<TestSensor>())
            .unwrap();

        assert_eq!(
            app.world_mut()
                .query::<&SensorState>()
                .iter(app.world())
                .next()
                .unwrap(),
            &SensorState(HashMap::from([(
                test_sensor_type_id,
                SensorValue::Bool(false)
            )]))
        );
    }

    #[test]
    fn updates_sensor_values() {
        let mut app = App::new();

        app.register_component_as::<dyn WorldSensor, TestSensor>()
            .add_systems(Update, collect_sensor_values);

        app.world_mut().spawn(TestSensor { active: false });

        app.update();

        app.world_mut()
            .run_system_once(|mut query: Query<&mut TestSensor>| {
                for mut sensor in query.iter_mut() {
                    sensor.active = true;
                }
            })
            .unwrap();

        app.update();

        let test_sensor_type_id = app
            .world()
            .components()
            .get_id(TypeId::of::<TestSensor>())
            .unwrap();

        assert_eq!(
            app.world_mut()
                .query::<&SensorState>()
                .iter(app.world())
                .next()
                .unwrap(),
            &SensorState(HashMap::from([(
                test_sensor_type_id,
                SensorValue::Bool(true)
            )]))
        );
    }
}
