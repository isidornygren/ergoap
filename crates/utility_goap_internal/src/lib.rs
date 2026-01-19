use bevy_app::{App, First, Plugin, PostUpdate};
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::world::World;
use plan::make_plan;
use world_sensor::collect_sensor_values;

use crate::auto_register::register_trait_types;

mod action_provider;
mod astar;
mod auto_register;
mod comparison;
mod current_action;
mod effect;
mod goal;
mod id_container;
mod plan;
mod sensor_state;
mod world_sensor;

pub(crate) use id_container::IdContainer;

pub use crate::action_provider::{ActionProvider, ActionProviderBuilder, ActionProviderTrait};
pub use crate::comparison::Comparison;
pub use crate::current_action::CurrentAction;
pub use crate::goal::Goal;
pub use crate::sensor_state::SensorState;
pub use crate::world_sensor::{SensorComparison, SensorEffect, SensorValue, WorldSensor};
pub use auto_register::{AutomaticTraitRegistrations, RegisterComponentAs};

pub mod __macro_exports {
    pub use bevy_ecs;
    pub use bevy_trait_query;
    pub use inventory;
}

pub struct UtilityGoapPlugin;

impl Plugin for UtilityGoapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (collect_sensor_values, make_plan).chain())
            .add_systems(First, |world: &mut World| register_trait_types(world));
    }
}
