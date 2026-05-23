use bevy::prelude::*;
use ergoap::prelude::*;

#[derive(Action, Clone)]
struct SomeAction;

#[derive(WorldSensor, Component)]
struct SomeSensor(bool);

#[derive(WorldSensor, Component)]
struct TargetSensor(pub Option<TargetValue>);

#[derive(Component)]
struct TargetComponent(bool);

fn update_target(
    mut query: Query<(Entity, &mut TargetSensor)>,
    target: Single<Entity, With<TargetComponent>>,
    transforms: Query<&Transform>,
) {
    for (entity, mut target_sensor) in &mut query {
        let distance = transforms
            .get_many([entity, *target])
            .map(|[transform, target_transform]| {
                transform.translation.distance(target_transform.translation)
            })
            .unwrap();

        target_sensor.0 = Some(TargetValue {
            entity: *target,
            is_close: distance < 0.1,
        });
    }
}

fn goto_target(
    mut query: Query<(Entity, &CurrentAction<GotoTarget>)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time<Virtual>>,
) {
    for (entity, goto_action) in &mut query {
        if let Ok([mut transform, target_transform]) =
            transforms.get_many_mut([entity, goto_action.target()])
        {
            let direction = target_transform.translation - transform.translation;
            let distance = direction.length();

            let movement = direction.normalize() * 50. * time.delta_secs();
            info!(
                "Going to target: {:?}, distance: {:?}",
                target_transform.translation, distance
            );

            if movement.length() < distance {
                transform.translation += movement;
            } else {
                transform.translation = target_transform.translation;
            }
        }
    }
}

fn update_sensor(
    mut query: Query<&mut SomeSensor>,
    target: Single<&TargetComponent, Changed<TargetComponent>>,
) {
    for mut sensor in &mut query {
        sensor.0 = target.0;
    }
}

fn execute_action(
    mut query: Query<&CurrentAction<SomeAction>>,
    mut target: Single<&mut TargetComponent>,
) {
    for _ in &mut query {
        info!("Turned target value on");
        target.0 = true;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        TargetSensor(None),
        SomeSensor(false),
        ActionProvider::new(SomeAction)
            .with_requirement(SomeSensor::is_false())
            .with_target::<TargetSensor>()
            .with_effect(SomeSensor::set(true)),
        Goal::from_requirement(SomeSensor::is_true()),
        Transform::from_xyz(-100., 0., 0.),
    ));

    commands.spawn((TargetComponent(false), Transform::from_xyz(100., 0., 0.)));
}

pub fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, ErgoapPlugin))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, execute_action)
        .add_systems(Update, goto_target)
        .add_systems(SensorUpdate, (update_target, update_sensor))
        .run();
}
