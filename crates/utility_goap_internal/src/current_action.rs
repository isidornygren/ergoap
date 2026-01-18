use std::any::{Any, TypeId};

use bevy_ecs::{
    component::{Component, ComponentId, Components},
    entity::Entity,
    lifecycle::HookContext,
    message::{Message, MessageReader},
    prelude::ReflectComponent,
    system::{Commands, Query},
    world::DeferredWorld,
};
use bevy_reflect::Reflect;
use bevy_trait_query::queryable;

#[queryable]
pub trait CurrentActionTrait {
    fn get_type_id(&self) -> TypeId {
        self.type_id()
    }
}

#[derive(Message)]
pub struct DespawnCurrentActions {
    pub entity: Entity,
    pub skip_component_id: Option<ComponentId>,
}

pub fn on_insert_current_action(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    world.commands().write_message(DespawnCurrentActions {
        skip_component_id: Some(component_id),
        entity,
    });
}

pub(crate) fn despawn_current_actions(
    mut reader: MessageReader<DespawnCurrentActions>,
    query: Query<&dyn CurrentActionTrait>,
    components: &Components,
    mut commands: Commands,
) {
    for message in reader.read() {
        if let Ok(current_actions) = query.get(message.entity) {
            for current_action in current_actions {
                if let Some(action_component_id) = components.get_id(current_action.get_type_id()) {
                    if message
                        .skip_component_id
                        .is_none_or(|id| id != action_component_id)
                    {
                        commands
                            .entity(message.entity)
                            .remove_by_id(action_component_id);
                    }
                }
            }
        }
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[component(on_insert=on_insert_current_action)]
pub struct CurrentAction<A: Reflect> {
    pub(crate) action: A,
}

impl<A: Reflect> CurrentActionTrait for CurrentAction<A> {}
