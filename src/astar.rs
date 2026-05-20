use bitvec::vec::BitVec;

use crate::{ActionProvider, goal::Goal, sensor_state::SensorState};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
};

#[derive(Debug, Clone, Default)]
struct Node {
    state: SensorState,
    bit_vec_state: BitVec,
    parent_index: Option<usize>,
    action_taken: Option<usize>,
    goal_cost: usize,
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

pub fn astar_plan(
    start_state: &SensorState,
    actions: &[ActionProvider],
    goal: &Goal,
) -> Option<Vec<usize>> {
    let mut closed_set = HashSet::with_capacity(actions.len());
    let mut all_nodes = Vec::new();

    let start_node = Node {
        state: start_state.clone(),
        bit_vec_state: start_state.bit_vec(),
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

        if closed_set.contains(&all_nodes[current_index].bit_vec_state) {
            continue;
        }
        closed_set.insert(all_nodes[current_index].bit_vec_state.clone());

        for (action_index, action) in actions.iter().enumerate() {
            if !action.preconditions_met(&all_nodes[current_index].state) {
                continue;
            }

            let mut new_state = all_nodes[current_index].state.clone();
            let mut new_bitvec = all_nodes[current_index].bit_vec_state.clone();
            action.apply(&mut new_state);
            action.apply_to_bitvec(&mut new_bitvec);

            if closed_set.contains(&new_bitvec) {
                continue;
            }
            let heuristic_cost = goal.distance(&new_state);

            let new_node = Node {
                state: new_state,
                bit_vec_state: new_bitvec,
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
