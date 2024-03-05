use std::collections::BinaryHeap;

use indicatif::ParallelProgressIterator;
use rand::seq::SliceRandom;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    ch::{
        contraction_helper::{ContractionHelper, ShortcutSearchResult},
        shortcut::Shortcut,
    },
    graphs::graph::Graph,
    graphs::types::VertexId,
};

use super::{deleted_neighbors::DeletedNeighbors, edge_difference::EdgeDifference, state::CHState};

pub trait PriorityTerm {
    /// Gets the priority of node v in the graph
    fn priority(
        &self,
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32;

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
        queue.register(190, EdgeDifference::new(&graph));
        queue.register(120, DeletedNeighbors::new(&graph));
        queue.initialize(graph);
        queue
    }

    fn register(&mut self, weight: i32, term: impl PriorityTerm + 'static + Sync) {
        self.priority_terms.push((weight, Box::new(term)));
    }

    // Lazy poping the vertex with minimum priority.
    pub fn pop(&mut self, graph: &Graph) -> Option<(VertexId, Vec<Shortcut>)> {
        while let Some(mut state) = self.queue.pop() {
            // If current priority is greater than minimum priority, then repush state with updated
            // priority.
            let priority_shortcuts = self.get_priority_and_shortcuts_mut(state.vertex, graph);
            if priority_shortcuts.0 > state.priority {
                state.priority = priority_shortcuts.0;
                self.queue.push(state);
                continue;
            }

            self.update_before_contraction(state.vertex, graph);
            return Some((state.vertex, priority_shortcuts.1.shortcuts));
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

    pub fn get_priority_and_shortcuts_mut(
        &mut self,
        vertex: VertexId,
        graph: &Graph,
    ) -> (i32, ShortcutSearchResult) {
        let shortcuts_results = self.get_shortcuts(vertex, graph);

        self.get_priority(graph, shortcuts_results, vertex)
    }

    fn get_shortcuts(&mut self, vertex: u32, graph: &Graph) -> ShortcutSearchResult {
        let shortcut_generator = ContractionHelper::new(graph, 100);
        let shortcuts = shortcut_generator.get_shortcuts(vertex);
        shortcuts
    }

    pub fn get_priority_and_shortcuts_init(
        &self,
        vertex: VertexId,
        graph: &Graph,
    ) -> (i32, ShortcutSearchResult) {
        let shortcut_generator = ContractionHelper::new(graph, 100);
        let shortcuts_results = shortcut_generator.get_shortcuts(vertex);

        self.get_priority(graph, shortcuts_results, vertex)
    }

    fn initialize(&mut self, graph: &Graph) {
        let mut order: Vec<u32> = (0..graph.number_of_vertices()).map(|x| x as u32).collect();
        order.shuffle(&mut rand::thread_rng());

        let vertex_and_priority_and_shortcuts: Vec<_> = order
            .par_iter()
            .progress()
            .map(|&vertex| (vertex, self.get_priority_and_shortcuts_init(vertex, graph)))
            .collect();

        self.queue = vertex_and_priority_and_shortcuts
            .into_iter()
            .map(|(vertex, (priority, _))| CHState { vertex, priority })
            .collect();
    }

    fn get_priority(
        &self,
        graph: &Graph,
        shortcuts_results: ShortcutSearchResult,
        vertex: u32,
    ) -> (i32, ShortcutSearchResult) {
        let priorities: Vec<i32> = self
            .priority_terms
            .iter()
            .map(|priority_term| {
                priority_term.0 * priority_term.1.priority(vertex, graph, &shortcuts_results)
            })
            .collect();

        (priorities.iter().sum::<i32>(), shortcuts_results)
    }
}
