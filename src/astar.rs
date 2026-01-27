//! A* pathfinding algorithm for GOAP planning.
//!
//! This module implements the A* search algorithm to find optimal action sequences
//! that satisfy a goal. The algorithm explores possible action sequences, evaluating
//! them based on actual cost and heuristic distance to the goal.

use crate::{action_provider::ActionProviderTrait, goal::Goal, sensor_state::SensorState};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
};

/// A search node representing a world state and the path taken to reach it.
///
/// Nodes are used internally by the A* algorithm to track explored states
/// and reconstruct the optimal action sequence when a goal is found.
#[derive(Debug, Clone, Default)]
struct Node {
    /// The world state at this node
    state: SensorState,
    /// Index of the parent node in the search tree
    parent_index: Option<usize>,
    /// Index of the action taken to reach this node
    action_taken: Option<usize>,
    /// Actual cost from start to this node
    goal_cost: usize,
    /// Estimated cost from this node to goal (heuristic)
    heuristic_cost: usize,
}

impl Node {
    #[must_use]
    const fn cost(&self) -> usize {
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

/// Calculate a hash for a hashable value.
///
/// Used to efficiently check if a state has already been explored during A* search.
fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};

    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);

    hasher.finish()
}

/// Reconstruct the action sequence from start to goal by backtracking through nodes.
///
/// # Arguments
///
/// * `nodes` - All nodes explored during the search
/// * `goal_index` - Index of the node that satisfied the goal
///
/// # Returns
///
/// A vector of action indices representing the optimal path from start to goal.
fn reconstruct_path(nodes: &[Node], goal_index: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current_index = Some(goal_index);

    while let Some(index) = current_index {
        let node = &nodes[index];
        if let Some(action) = node.action_taken {
            path.push(action);
        }
        current_index = node.parent_index;
    }

    path.reverse();
    path
}

/// Find an optimal action sequence to achieve a goal using A* search.
///
/// This function implements the A* pathfinding algorithm to find the lowest-cost
/// sequence of actions that satisfies the given goal. It explores possible action
/// sequences, prioritizing paths with lower combined actual and heuristic costs.
///
/// # Arguments
///
/// * `start_state` - The initial world state to plan from
/// * `actions` - Available actions that can be performed
/// * `goal` - The goal to achieve
///
/// # Returns
///
/// * `Some(Vec<usize>)` - Indices of actions to perform in sequence to reach the goal
/// * `None` - No valid plan exists to achieve the goal
///
/// # Algorithm
///
/// 1. Start with the initial state
/// 2. Explore actions whose preconditions are met
/// 3. Simulate applying each action to generate new states
/// 4. Prioritize states with lower f-cost (g + h)
/// 5. Return the first path that satisfies the goal
/// 6. Skip already-explored states to avoid cycles
pub fn astar_plan(
    start_state: &SensorState,
    actions: &Vec<&dyn ActionProviderTrait>,
    goal: &Goal,
) -> Option<Vec<usize>> {
    let mut closed_set = HashSet::with_capacity(actions.len());
    let mut all_nodes = Vec::new();

    let start_node = Node {
        state: start_state.clone(),
        heuristic_cost: goal.distance(start_state),
        ..Default::default()
    };

    all_nodes.push(start_node);

    let mut open_set = BinaryHeap::with_capacity(actions.len());
    open_set.push((std::cmp::Reverse(all_nodes[0].cost()), 0));

    while let Some((_, current_index)) = open_set.pop() {
        if goal.is_satisfied(&all_nodes[current_index].state) {
            let path = reconstruct_path(&all_nodes, current_index);
            return if path.is_empty() { None } else { Some(path) };
        }

        let state_hash = calculate_hash(&all_nodes[current_index].state);
        if closed_set.contains(&state_hash) {
            continue;
        }
        closed_set.insert(state_hash);

        for (action_index, action) in actions.iter().enumerate() {
            if !action.preconditions_met(&all_nodes[current_index].state) {
                continue;
            }

            let mut new_state = all_nodes[current_index].state.clone();
            action.apply(&mut new_state);

            let new_state_hash = calculate_hash(&new_state);
            if closed_set.contains(&new_state_hash) {
                continue;
            }
            let heuristic_cost = goal.distance(&new_state);

            let new_node = Node {
                state: new_state,
                parent_index: Some(current_index),
                action_taken: Some(action_index),
                goal_cost: all_nodes[current_index].goal_cost + action.cost(),
                heuristic_cost,
            };

            let new_cost = new_node.cost();
            let new_index = all_nodes.len();
            all_nodes.push(new_node);
            open_set.push((std::cmp::Reverse(new_cost), new_index));
        }
    }

    None
}
