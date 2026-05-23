use bevy::{ecs::component::Mutable, prelude::*};
use bevy_ecs::query::QueryFilter;
use ergoap::prelude::*;

trait SetSensorValue<T> {
    fn set_value(&mut self, value: T);
    fn clear(&mut self);
    fn max_distance(&self) -> Option<f32> {
        None
    }
}

#[derive(Component, WorldSensor)]
struct LampSensor(bool);

#[derive(Component, WorldSensor, Default)]
struct LampTarget(Option<TargetValue>);

impl SetSensorValue<TargetValue> for LampTarget {
    fn set_value(&mut self, value: TargetValue) {
        self.0 = Some(value);
    }
    fn clear(&mut self) {
        self.0 = None;
    }
}

#[derive(Component, WorldSensor, Default)]
struct SleepTarget(Option<TargetValue>);

#[derive(Clone, Action)]
struct ToggleLampAction {
    to: bool,
}

#[derive(Clone, Action)]
struct EatFoodAction;

#[derive(Clone, Action)]
struct RunAway;

#[derive(Clone, Action)]
struct Idle;

#[derive(Component)]
struct Lamp {
    status: bool,
    material_on: Handle<ColorMaterial>,
    material_off: Handle<ColorMaterial>,
}

#[derive(Component, WorldSensor, Default)]
struct FoodTarget(Option<TargetValue>);

impl SetSensorValue<TargetValue> for FoodTarget {
    fn set_value(&mut self, value: TargetValue) {
        self.0 = Some(value);
    }
    fn clear(&mut self) {
        self.0 = None;
    }
}

#[derive(Component, WorldSensor, Default)]
struct EnemyTarget(Option<TargetValue>);

impl SetSensorValue<TargetValue> for EnemyTarget {
    fn set_value(&mut self, value: TargetValue) {
        self.0 = Some(value);
    }

    fn clear(&mut self) {
        self.0 = None;
    }

    fn max_distance(&self) -> Option<f32> {
        Some(50.0)
    }
}

#[derive(Component)]
struct BlueTeam;

#[derive(Component)]
struct RedTeam;

#[derive(Component)]
struct Food;

#[derive(Component, Debug)]
struct Hunger(f32);

#[derive(Component, Debug, WorldSensor)]
struct IsHungry(bool);

#[derive(Scorer, Default)]
struct HungryScorer(f32);

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
    for mut sensor in &mut query {
        sensor.0 = lamp.status;
    }
}

fn update_hunger(mut query: Query<(&mut Hunger, &mut IsHungry)>, time: Res<Time<Virtual>>) {
    for (mut hunger, mut is_hungry) in &mut query {
        hunger.0 += time.delta_secs();
        is_hungry.0 = hunger.0 > 10.;
    }
}

fn update_closest_target<
    TargetSensor: SetSensorValue<TargetValue> + Component<Mutability = Mutable>,
    TargetFilter: QueryFilter,
    EntityFilter: QueryFilter,
>(
    mut query: Query<(Entity, &mut TargetSensor), EntityFilter>,
    target: Query<Entity, TargetFilter>,
    transforms: Query<&Transform>,
) {
    for (entity, mut target_sensor) in &mut query {
        if let Some((target, distance)) = target
            .iter()
            .map(|target_entity| {
                let distance = transforms
                    .get_many([entity, target_entity])
                    .map(|[transform, target_transform]| {
                        transform.translation.distance(target_transform.translation)
                    })
                    .expect("Could not get distance between target");
                (target_entity, distance)
            })
            .reduce(
                |(entity_a, a), (entity_b, b)| {
                    if a < b { (entity_a, a) } else { (entity_b, b) }
                },
            )
            && target_sensor
                .max_distance()
                .is_none_or(|max_distance| max_distance > distance)
        {
            target_sensor.set_value(TargetValue {
                entity: target,
                is_close: distance < 0.1,
            });
        } else {
            target_sensor.clear();
        }
    }
}

fn eat_food(
    mut query: Query<(&FoodTarget, &mut Hunger), With<CurrentAction<EatFoodAction>>>,
    mut commands: Commands,
) {
    for (food_target, mut weariness) in &mut query {
        if let Some(target) = food_target.0 {
            weariness.0 = 0.0;
            commands.entity(target.entity).despawn();
        }
    }
}

fn goto(
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

            let walk_speed = 50.0;

            if distance >= 0.1 {
                let movement = direction.normalize() * walk_speed * time.delta_secs();

                if movement.length() < distance {
                    transform.translation += movement;
                } else {
                    transform.translation = target_transform.translation;
                }
            }
        }
    }
}

fn idle(mut query: Query<&mut Transform, With<CurrentAction<Idle>>>, time: Res<Time<Virtual>>) {
    for mut transform in &mut query {
        let walk_speed = 50.0;

        let movement = Vec3::new(1.0, 0.0, 0.0) * walk_speed * time.delta_secs();

        transform.translation += movement;
    }
}

fn run_away<Target: Component + WorldSensorValue<Option<TargetValue>>>(
    mut query: Query<(Entity, &Target), With<CurrentAction<RunAway>>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time<Virtual>>,
) {
    for (entity, target) in &mut query {
        if let Some(target_entity) = target.value()
            && let Ok([mut transform, target_transform]) =
                transforms.get_many_mut([entity, target_entity.entity])
        {
            let direction = transform.translation - target_transform.translation;
            let distance = direction.length();
            let walk_speed = 50.0;

            if distance > 0.0 {
                let movement = direction.normalize() * walk_speed * time.delta_secs();
                transform.translation += movement;
            }
        }
    }
}

fn sensor_bundle() -> impl Bundle {
    (
        LampSensor(false),
        LampTarget::default(),
        FoodTarget::default(),
        EnemyTarget::default(),
        IsHungry(false),
        Hunger(0.0),
    )
}

fn actor_bundle(
    lamp_value: bool,
    mesh: Handle<Mesh>,
    color: Handle<ColorMaterial>,
    position: Vec3,
) -> impl Bundle {
    (
        SensorState::default(),
        Goal::from_requirement(LampSensor::equal(lamp_value)),
        ActionProvider::new(RunAway)
            .with_requirement(EnemyTarget::is_some())
            .with_effect(EnemyTarget::set(None)),
        ActionProvider::new(ToggleLampAction { to: lamp_value })
            .with_effect(LampSensor::set(lamp_value))
            .with_requirement(IsHungry::is_false())
            .with_requirement(LampSensor::equal(!lamp_value))
            .with_requirement(EnemyTarget::is_none())
            .with_target::<LampTarget>(),
        ActionProvider::new(EatFoodAction)
            .with_effect(IsHungry::set(false))
            .with_requirement(IsHungry::is_true())
            .with_requirement(EnemyTarget::is_none())
            .with_target::<FoodTarget>(),
        GoalProvider::new(HungryScorer::default()).with_requirement(LampSensor::equal(lamp_value)),
        Otherwise::new(Idle),
        Picker::highest_scorer(),
        Transform::from_translation(position),
        Mesh2d(mesh),
        MeshMaterial2d(color),
    )
}

fn spawn_actors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle::new(10.0));
    commands.spawn((
        sensor_bundle(),
        actor_bundle(
            false,
            mesh.clone(),
            materials.add(Color::linear_rgb(1.0, 0.0, 0.0)),
            Vec3::new(100., 0., 1.),
        ),
        RedTeam,
    ));
    commands.spawn((
        sensor_bundle(),
        actor_bundle(
            true,
            mesh,
            materials.add(Color::linear_rgb(0.0, 1.0, 0.0)),
            Vec3::new(-100., 0., 1.),
        ),
        BlueTeam,
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

#[derive(Resource)]
struct FoodSpawnTimer {
    timer: Timer,
}

struct Lcg {
    state: u32,
}

impl Lcg {
    const fn next(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.state
    }

    fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let normalized = self.next() as f32 / u32::MAX as f32;
        normalized.mul_add(max - min, min)
    }
}

fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut food_spawn_timer: ResMut<FoodSpawnTimer>,
    time: Res<Time<Virtual>>,
) {
    let mut rng = Lcg {
        #[allow(clippy::cast_possible_truncation)]
        state: time.elapsed().as_nanos() as u32,
    };
    food_spawn_timer.timer.tick(time.delta());
    if food_spawn_timer.timer.is_finished() {
        let x = rng.range_f32(-300.0, 300.0);
        let y = rng.range_f32(-300.0, 300.0);

        commands.spawn((
            Food,
            Transform::from_xyz(x, y, 0.),
            Mesh2d(meshes.add(Circle::new(5.0))),
            MeshMaterial2d(materials.add(Color::linear_rgb(0.0, 0.0, 1.0))),
        ));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, ErgoapPlugin))
        .insert_resource(FoodSpawnTimer {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        })
        .add_systems(Startup, (setup, spawn_actors, spawn_lamp))
        .add_systems(Update, (goto, idle, run_away::<EnemyTarget>, spawn_food))
        .add_systems(FixedUpdate, (toggle_sensor, eat_food))
        .add_systems(
            SensorUpdate,
            (
                update_hunger,
                update_lamp_sensor,
                update_closest_target::<LampTarget, With<Lamp>, ()>,
                update_closest_target::<FoodTarget, With<Food>, ()>,
                update_closest_target::<EnemyTarget, With<BlueTeam>, With<RedTeam>>,
                update_closest_target::<EnemyTarget, With<RedTeam>, With<BlueTeam>>,
            ),
        )
        .run();
}
