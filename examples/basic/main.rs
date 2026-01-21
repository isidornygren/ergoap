use bevy::{prelude::*, sprite_render::Wireframe2dPlugin};
use utility_goap::prelude::*;

#[derive(Component, WorldSensor)]
struct LampSensor(bool);

#[derive(Component, WorldSensor, Debug)]
struct CloseToLampSensor(bool);

#[derive(Component, TargetSensor)]
struct ClosestTarget(Option<Entity>);

#[derive(Clone, Reflect, Action)]
struct ToggleLampAction {
    to: bool,
}

#[derive(Clone, Reflect, Action)]
struct GoToLampAction;

#[derive(Component)]
struct Lamp {
    status: bool,
    material_on: Handle<ColorMaterial>,
    material_off: Handle<ColorMaterial>,
}

#[derive(Component, WorldSensor, Debug)]
struct Weariness(f32);

#[derive(Clone, Reflect, Action)]
struct Sleep;

fn toggle_sensor(
    query: Query<&CurrentAction<ToggleLampAction>>,
    mut lamp: Single<(&mut Lamp, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    for current_action in query {
        lamp.0.status = current_action.to;
        lamp.1.0 = if current_action.to {
            lamp.0.material_on.clone()
        } else {
            lamp.0.material_off.clone()
        };
    }
}

fn update_lamp_sensor(lamp: Single<&Lamp, Changed<Lamp>>, mut query: Query<&mut LampSensor>) {
    for mut sensor in query.iter_mut() {
        sensor.0 = lamp.status;
    }
}

fn update_close_to_lamp_sensor(
    lamp_transform: Single<&Transform, With<Lamp>>,
    mut query: Query<(&Transform, &mut CloseToLampSensor)>,
) {
    for (transform, mut close_to_lamp_sensor) in query.iter_mut() {
        let distance = transform.translation.distance(lamp_transform.translation);
        let is_close_to = distance < 1.0;

        if close_to_lamp_sensor.0 != is_close_to {
            close_to_lamp_sensor.0 = is_close_to;
        }
    }
}

fn update_weariness(
    mut query: Query<(&mut Weariness, Option<&CurrentAction<Sleep>>)>,
    time: Res<Time<Virtual>>,
) {
    for (mut hunger, maybe_sleep) in query.iter_mut() {
        hunger.0 += if maybe_sleep.is_some() {
            time.delta_secs() * -1.
        } else {
            time.delta_secs()
        }
    }
}

fn goto<C: Component, A: Send + Sync + 'static>(
    lamp_transform: Single<&Transform, With<C>>,
    mut query: Query<&mut Transform, (With<CurrentAction<A>>, Without<C>)>,
    time: Res<Time<Virtual>>,
) {
    for mut transform in query.iter_mut() {
        let direction = lamp_transform.translation - transform.translation;
        let distance = direction.length();

        let walk_speed = 50.0;

        if distance >= 0.1 {
            let movement = direction.normalize() * walk_speed * time.delta_secs();

            if movement.length() < distance {
                transform.translation += movement;
            } else {
                transform.translation = lamp_transform.translation;
            }
        }
    }
}

fn spawn_actor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        LampSensor(true),
        CloseToLampSensor(false),
        Weariness(0.0),
        Goal::from_requirement(LampSensor::equal(false)),
        ActionProvider::new(ToggleLampAction { to: false })
            .with_effect(LampSensor::set(false))
            .with_requirement(Weariness::less_than(10.))
            .with_requirement(LampSensor::equal(true))
            .with_requirement(CloseToLampSensor::equal(true))
            .with_target::<ClosestTarget>()
            .with_cost(1),
        ActionProvider::new(GoToLampAction)
            .with_effect(CloseToLampSensor::set(true))
            .with_cost(1),
        Transform::from_xyz(-200., 0., 0.),
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.0, 1.0, 0.0))),
    ));
}

fn spawn_anti_actor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        LampSensor(false),
        CloseToLampSensor(false),
        Weariness(0.0),
        Goal::from_requirement(LampSensor::equal(true)),
        ActionProvider::new(ToggleLampAction { to: true })
            .with_effect(LampSensor::set(true))
            .with_requirement(Weariness::less_than(10.))
            .with_requirement(LampSensor::equal(false))
            .with_requirement(CloseToLampSensor::equal(true))
            .with_cost(1),
        ActionProvider::new(GoToLampAction)
            .with_effect(CloseToLampSensor::set(true))
            .with_cost(1),
        Transform::from_xyz(200., 0., 0.),
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(Color::linear_rgb(1.0, 0.0, 0.0))),
    ));
}

fn spawn_lamp(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_on = materials.add(Color::linear_rgb(1.0, 1.0, 1.0));
    let material_off = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    commands.spawn((
        Lamp {
            status: false,
            material_off: material_off.clone(),
            material_on,
        },
        Transform::from_translation(Vec3::splat(0.0)),
        Mesh2d(meshes.add(Circle::new(15.0))),
        MeshMaterial2d(material_off),
    ));
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        UtilityGoapPlugin,
        Wireframe2dPlugin::default(),
    ))
    .add_systems(Startup, (setup, spawn_actor, spawn_anti_actor, spawn_lamp))
    .add_systems(Update, goto::<Lamp, GoToLampAction>)
    .add_systems(
        FixedUpdate,
        (
            toggle_sensor,
            update_weariness,
            update_lamp_sensor,
            update_close_to_lamp_sensor,
        ),
    );

    app.run();
}
