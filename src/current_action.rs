//! Current action management for executing planned actions.
//!
//! This module provides components and systems for managing the currently executing action
//! on an entity. It handles action insertion, removal, and automatic cleanup of previous actions.

use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    component::{Component, ComponentId},
    lifecycle::HookContext,
    prelude::ReflectComponent,
    system::EntityCommands,
    world::{DeferredWorld, EntityWorldMut},
};
use bevy_reflect::Reflect;

use crate::ActionProviderTrait;
#[cfg(feature = "target")]
use crate::GotoTarget;

/// Hook function called when a [`CurrentAction`] component is inserted.
///
/// This hook automatically manages the [`CurrentActionRef`] component, which tracks
/// the component ID of the current action. It removes the previous action component
/// if a different action was active.
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

/// Internal component tracking the component ID of the current action.
///
/// This component is automatically managed by the [`CurrentAction`] insertion hook
/// and should not be modified directly.
#[derive(Component, Debug)]
pub struct CurrentActionRef(ComponentId);

/// Component representing the currently executing action on an entity.
///
/// This component wraps an action type and provides dereferencing to access the action.
/// When inserted, it automatically removes any previous action and updates the action reference.
///
/// # Type Parameters
///
/// * `A` - The action type being executed
///
/// # Example
///
/// ```ignore
/// use utility_goap::prelude::*;
///
/// #[derive(Clone, Debug)]
/// struct MoveAction {
///     target: Vec3,
/// }
///
/// // Insert a current action
/// commands.entity(entity).insert(CurrentAction {
///     action: MoveAction { target: Vec3::ZERO },
/// });
/// ```
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

/// Extension trait for [`EntityCommands`] to manage current actions.
///
/// This trait provides convenient methods for spawning, inserting, and removing
/// current actions on entities. It handles the complexity of target-based actions
/// and automatic action cleanup.
pub trait CurrentActionCommands {
    /// Spawn a current action with automatic target handling.
    ///
    /// If the `target` feature is enabled and the action requires a target,
    /// this will insert a [`GotoTarget`] action if the target is not yet close.
    /// Otherwise, it inserts the action directly.
    ///
    /// [`GotoTarget`]: crate::GotoTarget
    fn spawn_current_action(&mut self, action: Box<dyn ActionProviderTrait>);

    /// Remove the current action from the entity.
    ///
    /// This clears both the action component and the internal action reference.
    fn despawn_current_action(&mut self);

    /// Insert an action as the current action.
    ///
    /// This is a lower-level method that directly inserts the action without
    /// target handling. Prefer [`spawn_current_action`] for automatic target management.
    ///
    /// [`spawn_current_action`]: CurrentActionCommands::spawn_current_action
    fn insert_action(&mut self, action: &dyn ActionProviderTrait);
}

impl CurrentActionCommands for EntityCommands<'_> {
    fn insert_action(&mut self, action: &dyn ActionProviderTrait) {
        let cloned_action = action.clone_box();
        self.queue(move |mut entity_world: EntityWorldMut| {
            cloned_action.insert_current_action(&mut entity_world);
        });
    }

    fn spawn_current_action(&mut self, action: Box<dyn ActionProviderTrait>) {
        #[cfg(feature = "target")]
        if let Some(target) = *action.target() {
            self.queue(move |mut entity_world: EntityWorldMut| {
                use crate::{SensorState, SensorValue, world_sensor::TargetValue};

                let sensor_state = entity_world
                    .get::<SensorState>()
                    .expect("Could not get sensor state");
                let current_target = sensor_state
                    .get(&target)
                    .expect("Could not get current target");
                let (entity, is_close) = match current_target {
                    SensorValue::Target(Some(TargetValue { entity, is_close })) => {
                        Some((entity, is_close))
                    }
                    _ => None,
                }
                .expect("Current target not the correct value");
                if *is_close {
                    action.insert_current_action(&mut entity_world);
                } else {
                    entity_world.insert(CurrentAction {
                        action: GotoTarget {
                            next_action: Some(action),
                            target: *entity,
                        },
                    });
                }
            });
        } else {
            self.insert_action(&*action);
        }
        #[cfg(not(feature = "target"))]
        self.insert_action(&action);
    }

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
