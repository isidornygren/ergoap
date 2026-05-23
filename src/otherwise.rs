use bevy_ecs::component::Component;

use crate::{ActionProvider, ActionProviderTrait};

#[derive(Component)]
pub struct Otherwise {
    pub(crate) action: Box<dyn ActionProviderTrait>,
}

impl Otherwise {
    pub fn new<T: Component + Clone>(action: T) -> Self {
        Self {
            action: Box::new(ActionProvider {
                action,
                cost: 0,
                requirements: vec![],
                effects: vec![],
                #[cfg(feature = "target")]
                target: None,
            }),
        }
    }
}
