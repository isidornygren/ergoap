use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    prelude::ReflectComponent,
    system::EntityCommands,
    world::{DeferredWorld, EntityWorldMut},
};
use bevy_reflect::Reflect;

pub fn on_insert_current_action(
    mut world: DeferredWorld,
    HookContext {
        entity,
        component_id,
        ..
    }: HookContext,
) {
    if let Some(prev_action_ref) = world
        .entity(entity)
        .get::<CurrentActionRef>()
        .map(|action_ref| action_ref.0)
    {
        if prev_action_ref != component_id {
            world
                .commands()
                .entity(entity)
                .remove_by_id(prev_action_ref)
                .insert(CurrentActionRef(component_id));
        }
    } else {
        world
            .commands()
            .entity(entity)
            .insert(CurrentActionRef(component_id));
    }
}

#[derive(Component, Debug)]
pub struct CurrentActionRef(ComponentId);

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[component(on_insert=on_insert_current_action)]
pub struct CurrentAction<A> {
    pub(crate) action: A,
}

impl<A> Deref for CurrentAction<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.action
    }
}

impl<A> DerefMut for CurrentAction<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.action
    }
}

pub trait CurrentActionCommands {
    fn despawn_current_action(&mut self);
}

impl CurrentActionCommands for EntityCommands<'_> {
    fn despawn_current_action(&mut self) {
        self.queue(|mut entity_world: EntityWorldMut| {
            if let Some(component_id) = entity_world
                .get::<CurrentActionRef>()
                .map(|action_ref| action_ref.0)
            {
                entity_world.remove_by_id(component_id);
                entity_world.remove::<CurrentActionRef>();
            }
        });
    }
}
