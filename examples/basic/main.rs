use bevy::prelude::*;
use bevy_trait_query::RegisterExt;
use utility_goap::{UtilityGoapPlugin, prelude::*};

#[derive(Component)]
struct Sensor {
    pub active: bool,
}

impl WorldSensor for Sensor {
    fn sensor_value(&self) -> SensorValue {
        self.active.into()
    }
}

#[derive(Reflect, Clone, Default)]

struct TurnOffSensorAction;

fn turn_off_sensor(query: Query<(&mut Sensor, &CurrentAction<TurnOffSensorAction>)>) {
    for (mut sensor, current_action) in query {
        println!("Turned off sensor");
        sensor.active = false;
    }
}

fn spawn_actor(mut commands: Commands) {
    commands.spawn((
        Sensor { active: true },
        Goal::from_requirement(Requirement::equal::<Sensor>(false)),
        ActionProvider::new(TurnOffSensorAction)
            .with_effect(Effect::set::<Sensor>(false))
            .with_requirement(Requirement::equal::<Sensor>(true))
            .with_cost(1),
    ));
}

pub fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, UtilityGoapPlugin))
        .register_component_as::<dyn WorldSensor, Sensor>()
        .add_systems(Startup, spawn_actor)
        .add_systems(Update, turn_off_sensor);

    app.run();
}
