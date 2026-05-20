use bevy_ecs::component::Component;

use crate::ActionProvider;

#[derive(Component)]
pub struct Otherwise {
    pub(crate) action: ActionProvider,
}

impl Otherwise {
    pub fn new<T: Component + Clone + Send + Sync + 'static>(action: T) -> Self {
        Self {
            action: ActionProvider::from_action_with_cost(action, 0),
        }
    }
}
