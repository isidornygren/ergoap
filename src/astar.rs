use bevy_reflect::PartialReflect;

use crate::{action_provider::ActionProviderTrait, goal::Goal, sensor_state::SensorState};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
};

#[derive(Debug)]
struct Node {
    state: SensorState,
    actions: Vec<Box<dyn PartialReflect>>,
    goal_cost: usize,
    heuristic_cost: usize,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            actions: self.actions.iter().map(|a| a.to_dynamic()).collect(),
            goal_cost: self.goal_cost,
            heuristic_cost: self.heuristic_cost,
        }
    }
}

impl Node {
    fn cost(&self) -> usize {
        self.goal_cost + self.heuristic_cost
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost() == other.cost()
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost()
            .partial_cmp(&self.cost())
            .unwrap_or(Ordering::Equal)
    }
}

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};

    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);

    hasher.finish()
}

pub(crate) fn astar_plan(
    start_state: &SensorState,
    actions: Vec<&dyn ActionProviderTrait>,
    goal: &Goal,
) -> Option<Vec<Box<dyn PartialReflect>>> {
    let mut open_set = BinaryHeap::with_capacity(actions.len());
    let mut closed_set = HashSet::with_capacity(actions.len());

    let start_node = Node {
        state: start_state.clone(),
        actions: Vec::new(),
        goal_cost: 0,
        heuristic_cost: goal.distance(start_state),
    };

    open_set.push(start_node);

    while let Some(current) = open_set.pop() {
        if goal.is_satisfied(&current.state) {
            return if current.actions.is_empty() {
                None
            } else {
                Some(current.actions)
            };
        }

        let state_hash = calculate_hash(&current.state);
        if closed_set.contains(&state_hash) {
            continue;
        }
        closed_set.insert(state_hash);

        for action in actions.iter() {
            if !action.preconditions_met(&current.state) {
                continue;
            }

            let mut new_state = current.state.clone();
            action.apply(&mut new_state);

            let new_state_hash = calculate_hash(&new_state);
            if closed_set.contains(&new_state_hash) {
                continue;
            }

            let mut new_actions = current
                .actions
                .iter()
                .map(|action| action.to_dynamic())
                .collect::<Vec<_>>();
            new_actions.push(action.component().to_dynamic());

            open_set.push(Node {
                heuristic_cost: goal.distance(&new_state),
                state: new_state,
                actions: new_actions,
                goal_cost: current.goal_cost + action.cost(),
            });
        }
    }

    None
}
