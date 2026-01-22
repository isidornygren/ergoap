use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{Changed, Or},
    relationship::Relationship,
    system::{Commands, Query},
};

use crate::{
    ActionProviderTrait, CurrentAction, SensorState, SensorValue, TargetValue,
    current_action::CurrentActionCommands,
};

pub struct GotoTarget {
    pub(crate) action: Option<Box<dyn ActionProviderTrait>>,
    pub(crate) target: Entity,
}

impl CurrentAction<GotoTarget> {
    pub fn target(&self) -> Entity {
        self.action.target
    }
    pub fn goto_action(&self) -> &Option<Box<dyn ActionProviderTrait>> {
        &self.action.action
    }
}

impl Relationship for CurrentAction<GotoTarget> {
    type RelationshipTarget = TargetedBy;

    fn get(&self) -> Entity {
        self.action.target
    }

    fn from(entity: Entity) -> Self {
        Self {
            action: GotoTarget {
                action: None,
                target: entity,
            },
        }
    }

    fn set_risky(&mut self, entity: Entity) {
        self.action.target = entity
    }
}

#[derive(Component)]
#[relationship_target(relationship = CurrentAction<GotoTarget>, linked_spawn)]
pub struct TargetedBy(Vec<Entity>);

pub fn finish_goto(
    mut commands: Commands,
    query: Query<
        (Entity, &CurrentAction<GotoTarget>, &SensorState),
        Or<(Changed<CurrentAction<GotoTarget>>, Changed<SensorState>)>,
    >,
) {
    for (entity, goto_action, sensor_state) in &query {
        let sensor_state = sensor_state.get(
            goto_action
                .action
                .action
                .as_ref()
                .and_then(|a| a.target().as_ref())
                .unwrap(),
        );
        if let SensorValue::Target(Some(TargetValue { is_close, .. })) = sensor_state.unwrap() {
            if *is_close {
                commands.entity(entity).force_spawn_current_action(
                    goto_action
                        .goto_action()
                        .as_ref()
                        .expect("Tried to finish a goto action without an action"),
                );
            }
        }
    }
}
