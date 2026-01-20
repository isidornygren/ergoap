use bevy::prelude::*;
use utility_goap::prelude::*;

#[derive(Component, WorldSensor)]
struct Sensor(bool);

#[derive(Component, TargetSensor)]
struct ClosestTarget(Option<Entity>);

#[derive(Clone, Reflect, Action)]
struct TurnOffSensorAction;

fn turn_off_sensor(query: Query<&mut Sensor, With<CurrentAction<TurnOffSensorAction>>>) {
    for mut sensor in query {
        println!("Setting sensor state to false");
        sensor.0 = false;
    }
}

fn spawn_actor(mut commands: Commands) {
    commands.spawn((
        Sensor(true),
        Goal::from_requirement(Sensor::equal(false)),
        ActionProvider::new(TurnOffSensorAction)
            .with_effect(Sensor::set(false))
            .with_requirement(Sensor::equal(true))
            .with_target::<ClosestTarget>()
            .with_cost(1),
    ));
}

pub fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, UtilityGoapPlugin))
        .add_systems(Startup, spawn_actor)
        .add_systems(Update, turn_off_sensor);

    app.run();
}
