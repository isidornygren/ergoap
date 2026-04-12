use bevy::prelude::*;
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use ergoap::prelude::*;
use std::hint::black_box;

mod helpers;

#[derive(Action, Component, Clone)]
struct SomeAction;

fn spawn_action_provider(mut commands: Commands) {
    commands.spawn(ActionProvider::new(black_box(SomeAction)));
}

fn bench_spawn_action_provider(c: &mut Criterion) {
    c.bench_function("spawn_action_provider", |b| {
        b.iter_batched(
            || {
                let mut app = helpers::setup_app();
                app.add_systems(Update, spawn_action_provider);
                app
            },
            |mut app| {
                app.update();
            },
            BatchSize::SmallInput,
        );
    });
}

macro_rules! sensor_benchmark {
    ($count:literal) => {
        seq_macro::seq!(N in 0..$count {
            #[derive(WorldSensor, Component)]
            struct Sensor~N(bool);
        });

        seq_macro::seq!(N in 0..$count {
            #[derive(Action, Component, Clone)]
            struct Action~N;
        });

        fn setup_actions(app: &mut App){
            seq_macro::seq!(N in 0..$count {
                app.add_systems(
                    Update,
                    |mut query: Query<&mut Sensor~N, With<Action~N>>| {
                        for mut sensor in query.iter_mut() {
                            sensor.0 = true;
                        }
                    },
                );
            });
        }

        fn spawn_with_sensors(world: &mut World) -> EntityWorldMut<'_> {
            let mut entity_commands = world.spawn_empty();
            // Add an initial value here since it will push all values one step.
            let mut prev_values = vec![Sensor0::is_true()];
            seq_macro::seq!(N in 0..$count {
                prev_values.push(Sensor~N::is_true());
                entity_commands.insert(Sensor~N(false));
            });
            entity_commands.insert(
                ActionProvider::new(black_box(Action0))
                    .with_effect(Sensor0::set(true))
            );
            seq_macro::seq!(N in 1..$count {
                entity_commands.insert(
                    ActionProvider::new(black_box(Action~N))
                        .with_requirement(prev_values[N])
                        .with_requirement(Sensor~N::is_false())
                        .with_effect(Sensor~N::set(true))
                );
            });
            entity_commands
        }
    };
}

sensor_benchmark!(100);

fn bench_sensor_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("sensor_update");

    for entity_count in [1, 10, 50, 100] {
        for (path_name, sensor_depth) in [("short", 1), ("long", 99)] {
            group.bench_with_input(
                BenchmarkId::new(path_name, entity_count),
                &(entity_count, sensor_depth),
                |b, &(count, sensor_depth)| {
                    b.iter_batched(
                        || {
                            let mut app = helpers::setup_app();
                            setup_actions(&mut app);

                            for _ in 0..count {
                                let mut entity_commands = spawn_with_sensors(app.world_mut());
                                entity_commands.insert(Goal::from_requirement(
                                    if sensor_depth == 1 {
                                        Sensor1::is_true()
                                    } else {
                                        Sensor99::is_true()
                                    },
                                ));
                            }

                            app.world_mut().flush();
                            app.update();
                            app
                        },
                        |mut app| {
                            app.world_mut().run_schedule(Planning);
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_spawn_action_provider, bench_sensor_update);
criterion_main!(benches);
