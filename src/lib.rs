use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::schedule::IntoScheduleConfigs;
use plan::make_plan;
use world_sensor::collect_sensor_values;

mod action_provider;
mod astar;
mod effects;
mod goal;
mod plan;
mod requirement;
mod sensor_state;
mod world_sensor;

pub mod prelude {
    pub use crate::action_provider::{ActionProvider, ActionProviderBuilder, ActionProviderTrait};
    pub use crate::effects::Effect;
    pub use crate::goal::Goal;
    pub use crate::requirement::{Comparison, Requirement};
    pub use crate::sensor_state::SensorState;
    pub use crate::world_sensor::{SensorValue, WorldSensor};
}

pub struct UtilityGoapPlugin;

impl Plugin for UtilityGoapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (collect_sensor_values, make_plan).chain());
    }
}
