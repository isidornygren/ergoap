# ERGOAP

ERGOAP is an implementation of a GOAP (Goal Oriented Action Planning) system outlined in [this talk](https://www.youtube.com/watch?v=PaOLBOuyswI) with focus on ergonomics and ease-of-use, built for Bevy.

It is heavily inspired by the excellent [bevy-dogoap](https://github.com/victorb/dogoap) and [big-brain](https://codeberg.org/zkat/big-brain) which are also built for Bevy. What separates this implementation is the focus on having _all_ components as Bevy components in the actor (e.g not in a hierarchical structure) without relying on macros for generating the action composition. The rationale for this is that action providers and goals should be easily queried and modified in real time.

## Usage

### WorldSensors

A world sensor collates data from the world which can be used to interpret what actions or goals are valid. A `WorldSensor` can be constructed like so:

```rust
// As a newtype
#[derive(WorldSensor, Component)]
struct SomeSensor(bool);

// As a field
#[derive(WorldSensor, Component)]
struct FieldSensor {
    #[world_sensor]
    value: bool,
};

// Add a sensor like any other component
commands.spawn((SomeSensor(true), FieldSensor { value: true }));
```

You will then have to manually update the value in the sensor that the planner will consume.

### Actions

Actions are created using `ActionProvider`s which are generic components.

```rust
#[derive(Action, Clone)]
struct Perform;

let action_provider = ActionProvider::new(Perform)
    .with_requirement(SomeSensor::is_false())
    .with_effect(SomeSensor::set(true))
    // The cost of the action is by default 1, a higher cost
    .with_cost(2);

// Add an action provider like any other component, this will fail during runtime if
// the sensor component required by the action does not exist.
commands.spawn((SomeSensor(false), action_provider));
```

A quirk with `ActionProvider`s are that only one component of type `ActionProvider<SomeAction>` can exist tied to an entity, so this limits you to create a new struct for each action (within the same entity).

### Goals

The goal is the state that the planner should strive for. There can only exist one goal per entity (since it is a component).

```rust
let goal = Goal::from_requirement(SomeSensor::is_true());

commands.spawn(goal);
```

## Features

### `utility` (enabled by default)

Enable the utility feature in order to use `GoalProviders` which uses a `Score` in order to determine the current Goal.

### `target` (enabled by default)

The target feature enables the use of targets for `ActionProvider`s, these can be added by using the `with_target`, e.g

```rust
#[derive(WorldSensor, Component)]
TargetSensor(Option<TargetValue>);

ActionProvider::new(SomeAction)
    .with_target::<TargetSensor>();
```

An `ActionProvider` with a target produces a `CurrentAction<GotoTarget>` action when the TargetSensor value is populated and the `is_close` field equals `false`.
It is up to the user to set the `is_close` field which will produce the action within the `ActionProvider`. For a detailed example, see the `target` example.

## Limitations

### WASM is not currently supported

Due to the heavy use of the inventory crate for automatic trait type registration, WASM is not currently supported. This would probably not be an issue if Bevy upstreams `bevy-trait-query` with a similar implementation to the automatic registration of the trait types as with the reflect types. This could also be solved by registering the types in run-time instead, this is however unsupported currently in `bevy-trait-query` and I'm unsure of the performance overhead of this.

## Future work / Thoughts

I'd like to keep this library as minimal as possible and keep most of the implementation up to the user, the key points I'm not really sure of how to implement whilst keeping this in mind are:

### Remove the necessity to add all sensors manually

When you create a new actor with an `ActionProvider`, you have to manually add all the sensors that the `ActionProvider` consumes, e.g:

```rust
commands.spawn(
    SomeSensor(true),
    ActionProvider::new(SomeAction)
        .with_requirement(SomeSensor::is_true())
        .with_effect(SomeSensor::set(false))
);
```

I feel a bit unsure about how to weigh the ergonomics against clarity here, on one hand it would be nicer not to have to define the sensor state explicitly as it will _probably_ be set by a system anyway. On the other hand, the entity with the `ActionProvider` clearly states what value it instantiates with.

One way of solving this is to have all sensors implement `FromWorld` and follow a similar pattern to how Bevy does required components. Meaning that you would still be able to instantiate the sensors manually. If you choose not, then all sensors not explicitly created and owned by an `ActionProvider` will be instantiated using the `FromWorld` implementation.

### Remove bevy-trait-query

[bevy-trait-query](https://github.com/joseph-gio/bevy-trait-query) is an excellent library with little performance overhead and is used throughout this library in order to iterate over sensors, action providers and utility scorers.

However, with the current implementation, using this library means that all actions need to derive the `Action trait`, all sensors need to derive the `WorldSensor` trait and so on in order to properly be iterable as a trait. In the spirit of keeping things as minimal as possible I would prefer if it was not a requirement, especially for the Actions. However, due to the fact that the `bevy-trait-query` library is quite mature, [might be upstreamed](https://github.com/bevyengine/bevy/issues/15970) and me being too dumb to figure out how to do this, I'm not planning to do this.
