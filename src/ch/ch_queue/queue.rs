use std::collections::BinaryHeap;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rand::seq::SliceRandom;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};

use crate::{
    ch::{contraction_helper::ContractionHelper, shortcut::Shortcut},
    graphs::graph::Graph,
    graphs::types::VertexId,
};

use super::{
    cost_of_contraction::CostOfContraction, cost_of_queries::CostOfQueries,
    deleted_neighbors::DeletedNeighbors, state::CHState,
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
    vertex_shortcut: HashMap<VertexId, Vec<Shortcut>>,
}

impl CHQueue {
    pub fn new(graph: &Graph) -> Self {
        let queue = BinaryHeap::new();
        let priority_terms = Vec::new();
        let vertex_shortcut = HashMap::new();
        let mut queue = Self {
            queue,
            priority_terms,
            vertex_shortcut,
        };
        // queue.register(1, VoronoiRegion::new(&graph));
        queue.register(1, CostOfContraction::new(&graph));
        queue.register(120, DeletedNeighbors::new(&graph));
        // queue.register(1, CostOfQueries::new(&graph));
        queue.initialize(graph);
        queue
    }

    fn register(&mut self, weight: i32, term: impl PriorityTerm + 'static + Sync) {
        self.priority_terms.push((weight, Box::new(term)));
    }

    // Lazy poping the node with minimum priority.
    pub fn pop(&mut self, graph: &Graph) -> Option<(VertexId, Vec<Shortcut>)> {
        while let Some(mut state) = self.queue.pop() {
            // If current priority is greater than minimum priority, then repush state with updated
            // priority.
            let priority_shortcuts = self.get_priority_and_shortcuts_mut(state.vertex, graph);
            if priority_shortcuts.0 > state.priority {
                // println!("store");
                self.vertex_shortcut
                    .insert(state.vertex, priority_shortcuts.1);
                state.priority = priority_shortcuts.0;
                self.queue.push(state);
                continue;
            }

            for neighbor in graph.closed_neighborhood(state.vertex, 1) {
                self.vertex_shortcut.remove(&neighbor);
            }
            self.update_before_contraction(state.vertex, graph);
            return Some((state.vertex, priority_shortcuts.1));
        }
        None
    }

    // pub fn pop_vec(&mut self, graph: &Graph, max_size: u32) -> Option<Vec<VertexId>> {
    //     let mut neighbors = HashSet::new();
    //     let mut node_set = Vec::new();

    //     while let Some(mut state) = self.queue.pop() {
    //         // If current priority is greater than minimum priority, then repush state with updated
    //         // priority and try again.
    //         let current_priority = self.get_priority(state.vertex, graph);
    //         if current_priority > state.priority {
    //             state.priority = current_priority;
    //             self.queue.push(state);
    //             continue;
    //         }

    //         // If node is in set of neighbors, then repush state with updated priority and stop the
    //         // creation of the node set.
    //         if neighbors.contains(&state.vertex) || node_set.len() >= max_size as usize {
    //             state.priority = current_priority;
    //             self.queue.push(state);
    //             break;
    //         }

    //         self.update_before_contraction(state.vertex, graph);
    //         neighbors.extend(graph.open_neighborhood(state.vertex, 2));
    //         node_set.push(state.vertex);
    //     }

    //     if !node_set.is_empty() {
    //         return Some(node_set);
    //     }

    //     None
    // }

    /// Gets called just before a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        self.priority_terms
            .iter_mut()
            .for_each(|priority_term| priority_term.1.update_before_contraction(vertex, graph));
    }

    pub fn get_priority_and_shortcuts_mut(
        &mut self,
        vertex: VertexId,
        graph: &Graph,
    ) -> (i32, Vec<Shortcut>) {
        let priorities: Vec<i32> = self
            .priority_terms
            .iter()
            .map(|priority_term| priority_term.0 * priority_term.1.priority(vertex, graph))
            .collect();

        let shortcuts = self.get_shortcuts(vertex, graph);

        let number_of_edges =
            graph.in_edges[vertex as usize].len() + graph.out_edges[vertex as usize].len();

        let edge_difference = shortcuts.len() as i32 - number_of_edges as i32;

        (
            190 * edge_difference + priorities.iter().sum::<i32>(),
            shortcuts,
        )
    }

    fn get_shortcuts(&mut self, vertex: u32, graph: &Graph) -> Vec<Shortcut> {
        if let Some(this_shortcuts) = self.vertex_shortcut.remove(&vertex) {
            // println!("reuse");
            return this_shortcuts;
        } else {
            let shortcut_generator = ContractionHelper::new(graph, 10);
            let shortcuts = shortcut_generator.generate_shortcuts(vertex);
            shortcuts
        }
    }

    pub fn get_priority_and_shortcuts_init(
        &self,
        vertex: VertexId,
        graph: &Graph,
    ) -> (i32, Vec<Shortcut>) {
        let priorities: Vec<i32> = self
            .priority_terms
            .iter()
            .map(|priority_term| priority_term.0 * priority_term.1.priority(vertex, graph))
            .collect();

        let shortcut_generator = ContractionHelper::new(graph, 10);
        let shortcuts = shortcut_generator.generate_shortcuts(vertex);

        let number_of_edges =
            graph.in_edges[vertex as usize].len() + graph.out_edges[vertex as usize].len();

        let edge_difference = shortcuts.len() as i32 - number_of_edges as i32;

        (edge_difference + priorities.iter().sum::<i32>(), shortcuts)
    }

    // fn _update_queue(&mut self, graph: &Graph) {
    //     self.queue = self
    //         .queue
    //         .iter()
    //         .progress()
    //         .par_bridge()
    //         .map(|state| CHState {
    //             vertex: state.vertex,
    //             priority: self.get_priority_and_shortcuts(state.vertex, graph).0,
    //         })
    //         .collect();
    // }

    fn initialize(&mut self, graph: &Graph) {
        let mut order: Vec<u32> = (0..graph.out_edges.len()).map(|x| x as u32).collect();
        order.shuffle(&mut rand::thread_rng());

        let vertex_and_priority_and_shortcuts: Vec<_> = order
            .par_iter()
            .progress()
            .map(|&vertex| (vertex, self.get_priority_and_shortcuts_init(vertex, graph)))
            .collect();

        self.queue = vertex_and_priority_and_shortcuts
            .into_iter()
            .map(|(vertex, (priority, shortcuts))| {
                self.vertex_shortcut.insert(vertex, shortcuts);
                CHState { vertex, priority }
            })
            .collect();
    }
}
