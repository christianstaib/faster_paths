use std::collections::BinaryHeap;

use ahash::{HashSet, HashSetExt};
use indicatif::ProgressIterator;
use rand::seq::SliceRandom;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    ch::ch_queue::deleted_neighbors::DeletedNeighbors, graphs::graph::Graph,
    graphs::types::VertexId,
};

use super::{
    cost_of_queries::CostOfQueries, edge_difference::EdgeDifferencePriority, state::CHState,
    voronoi_region::VoronoiRegion,
};

pub trait PriorityTerm {
    /// Gets the priority of node v in the graph
    fn priority(&self, vertex: VertexId, graph: &Graph) -> i32;

    /// Gets called just BERFORE a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph);
}

pub struct CHQueue {
    queue: BinaryHeap<CHState>,
    priority_terms: Vec<(i32, Box<dyn PriorityTerm + Sync>)>,
}

impl CHQueue {
    pub fn new(graph: &Graph) -> Self {
        let queue = BinaryHeap::new();
        let priority_terms = Vec::new();
        let mut queue = Self {
            queue,
            priority_terms,
        };
        queue.register(1, EdgeDifferencePriority::new(&graph));
        queue.register(1, VoronoiRegion::new(&graph));
        queue.register(1, DeletedNeighbors::new(&graph));
        queue.register(25, CostOfQueries::new(&graph));
        queue.initialize(graph);
        queue
    }

    fn register(&mut self, weight: i32, term: impl PriorityTerm + 'static + Sync) {
        self.priority_terms.push((weight, Box::new(term)));
    }

    // Lazy poping the node with minimum priority.
    pub fn pop(&mut self, graph: &Graph) -> Option<u32> {
        while let Some(mut state) = self.queue.pop() {
            // If current priority is greater than minimum priority, then repush state with updated
            // priority.
            let current_priority = self.get_priority(state.vertex, graph);
            if current_priority > state.priority {
                state.priority = current_priority;
                self.queue.push(state);
                continue;
            }

            self.update_before_contraction(state.vertex, graph);
            return Some(state.vertex);
        }
        None
    }

    pub fn pop_vec(&mut self, graph: &Graph, max_size: u32) -> Option<Vec<u32>> {
        let mut neighbors = HashSet::new();
        let mut node_set = Vec::new();

        while let Some(mut state) = self.queue.pop() {
            // If current priority is greater than minimum priority, then repush state with updated
            // priority and try again.
            let current_priority = self.get_priority(state.vertex, graph);
            if current_priority > state.priority {
                state.priority = current_priority;
                self.queue.push(state);
                continue;
            }

            // If node is in set of neighbors, then repush state with updated priority and stop the
            // creation of the node set.
            if neighbors.contains(&state.vertex) || node_set.len() >= max_size as usize {
                state.priority = current_priority;
                self.queue.push(state);
                break;
            }

            self.update_before_contraction(state.vertex, graph);
            neighbors.extend(graph.open_neighborhood(state.vertex, 2));
            node_set.push(state.vertex);
        }

        if !node_set.is_empty() {
            return Some(node_set);
        }

        None
    }

    /// Gets called just before a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        self.priority_terms
            .iter_mut()
            .for_each(|priority_term| priority_term.1.update_before_contraction(vertex, graph));
    }

    pub fn get_priority(&self, v: VertexId, graph: &Graph) -> i32 {
        let priorities: Vec<i32> = self
            .priority_terms
            .iter()
            .map(|priority_term| priority_term.0 * priority_term.1.priority(v, graph))
            .collect();

        priorities.iter().sum()
    }

    fn _update_queue(&mut self, graph: &Graph) {
        self.queue = self
            .queue
            .iter()
            .progress()
            .par_bridge()
            .map(|state| CHState {
                vertex: state.vertex,
                priority: self.get_priority(state.vertex, graph),
            })
            .collect();
    }

    fn initialize(&mut self, graph: &Graph) {
        let mut order: Vec<u32> = (0..graph.out_edges.len()).map(|x| x as u32).collect();
        order.shuffle(&mut rand::thread_rng());

        self.queue = order
            .iter()
            .progress()
            .par_bridge()
            .map(|&v| CHState {
                vertex: v,
                priority: self.get_priority(v, graph),
            })
            .collect();
    }
}
