use std::any::TypeId;

use bevy::prelude::*;
use bevy_ecs::component::ComponentId;
use bevy_trait_query::RegisterExt;
use ergoap_macros::WorldSensor;

use crate::prelude::*;

#[derive(WorldSensor, Component)]
pub struct TestSensor(pub bool);

pub fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ErgoapPlugin));

    app.register_component_as::<dyn WorldSensor, TestSensor>();

    app
}

pub trait GetComponentId {
    fn get_component_id<T: Component>(&self) -> ComponentId;
}

impl GetComponentId for App {
    fn get_component_id<T: Component>(&self) -> ComponentId {
        self.world().components().get_id(TypeId::of::<T>()).unwrap()
    }
}
