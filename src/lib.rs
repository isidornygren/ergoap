use bevy_app::{App, First, FixedMainScheduleOrder, FixedUpdate, Plugin};
use bevy_ecs::schedule::{IntoScheduleConfigs, ScheduleLabel};
use bevy_ecs::world::World;
use plan::make_plan;
use world_sensor::collect_sensor_values;

use crate::auto_register::register_trait_types;
#[cfg(feature = "target")]
use crate::target::finish_goto;
#[cfg(feature = "target")]
use bevy_app::FixedPostUpdate;

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
#[cfg(feature = "target")]
mod target;
#[cfg(feature = "utility")]
mod utility;
mod world_sensor;

pub use crate::action_provider::{ActionProvider, ActionProviderBuilder, ActionProviderTrait};
pub use crate::comparison::Comparison;
pub use crate::current_action::CurrentAction;
pub use crate::goal::Goal;
pub use crate::sensor_state::SensorState;
#[cfg(feature = "target")]
pub use crate::world_sensor::TargetValue;
pub use crate::world_sensor::{
    SensorComparison, SensorComparisonBool, SensorComparisonOption, SensorEffect, SensorValue,
    WorldSensor, WorldSensorValue,
};
pub use auto_register::{AutomaticTraitRegistrations, RegisterComponentAs};
pub use id_container::IdContainer;
#[cfg(feature = "target")]
pub use target::GotoTarget;
#[cfg(feature = "utility")]
pub use utility::{GoalProvider, GoalProviderTrait, Score, Scorer};

pub mod __macro_exports {
    pub use bevy_ecs;
    pub use bevy_trait_query;
    pub use inventory;
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorUpdate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Planning;

pub struct ErgoapPlugin;

impl Plugin for ErgoapPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(SensorUpdate).init_schedule(Planning);
        {
            let mut main_schedule_order = app.world_mut().resource_mut::<FixedMainScheduleOrder>();
            main_schedule_order.insert_after(FixedUpdate, SensorUpdate);
            main_schedule_order.insert_after(SensorUpdate, Planning);
        }
        app.add_systems(Planning, (collect_sensor_values, make_plan).chain())
            .add_systems(First, |world: &mut World| register_trait_types(world));
        #[cfg(feature = "target")]
        app.add_systems(FixedPostUpdate, finish_goto);
        #[cfg(feature = "utility")]
        app.add_plugins(utility::plugin);
    }
}

pub mod prelude {
    #[cfg(feature = "utility")]
    pub use crate::utility::{
        GoalProvider, GoalProviderBuilder, GoalProviderTrait, Picker, Score, Scorer,
    };
    pub use crate::{
        ErgoapPlugin, Planning, SensorUpdate,
        action_provider::{ActionProvider, ActionProviderBuilder, ActionProviderTrait},
        auto_register::{AutomaticTraitRegistrations, RegisterComponentAs},
        comparison::Comparison,
        current_action::CurrentAction,
        goal::Goal,
        sensor_state::SensorState,
        world_sensor::{
            SensorComparison, SensorComparisonBool, SensorComparisonOption, SensorEffect,
            SensorValue, WorldSensor, WorldSensorValue,
        },
    };
    #[cfg(feature = "target")]
    pub use crate::{target::GotoTarget, world_sensor::TargetValue};
    pub use ergoap_macros::*;
}
