use std::collections::BinaryHeap;

use indicatif::ParallelProgressIterator;
use rand::seq::SliceRandom;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    ch::{
        contraction_helper::{ContractionHelper, ShortcutSearchResult},
        shortcut::Shortcut,
    },
    graphs::graph::Graph,
    graphs::types::VertexId,
};

use super::{
    cost_of_queries::CostOfQueries, deleted_neighbors::DeletedNeighbors,
    edge_difference::EdgeDifference, priority_function::PriorityFunction, state::CHState,
};

pub struct CHQueue {
    queue: BinaryHeap<CHState>,
    priority_terms: Vec<(i32, Box<dyn PriorityFunction + Sync>)>,
}

impl CHQueue {
    pub fn new(graph: &Graph, priority_functions_letters: &str) -> Self {
        let queue = BinaryHeap::new();
        let priority_functions = Vec::new();
        let mut queue = Self {
            queue,
            priority_terms: priority_functions,
        };
        for letter in priority_functions_letters.chars() {
            match letter {
                'E' => queue.register(1, EdgeDifference::new(&graph)),
                'D' => queue.register(1, DeletedNeighbors::new(&graph)),
                'C' => queue.register(1, CostOfQueries::new(&graph)),
                _ => panic!("letter not recognized"),
            }
        }
        queue.initialize(graph);
        queue
    }

    fn register(
        &mut self,
        coefficent: i32,
        priority_function: impl PriorityFunction + 'static + Sync,
    ) {
        self.priority_terms
            .push((coefficent, Box::new(priority_function)));
    }

    // Lazy poping the vertex with minimum priority.
    pub fn pop(&mut self, graph: &Graph) -> Option<(VertexId, Vec<Shortcut>)> {
        while let Some(mut state) = self.queue.pop() {
            // If current priority is greater than minimum priority, then repush state with updated
            // priority.
            let (priority, shortcuts) = self.priority_and_shortcuts(state.vertex, graph);
            if priority > state.priority {
                state.priority = priority;
                self.queue.push(state);
                continue;
            }

            self.update_before_contraction(state.vertex, graph);
            return Some((state.vertex, shortcuts));
        }
        None
    }

    /// Gets called just before a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        self.priority_terms
            .iter_mut()
            .for_each(|(_, priority_function)| priority_function.update(vertex, graph));
    }

    pub fn priority_and_shortcuts(&self, vertex: VertexId, graph: &Graph) -> (i32, Vec<Shortcut>) {
        let shortcut_generator = ContractionHelper::new(graph, 100);
        let shortcuts_results = shortcut_generator.get_shortcuts(vertex);
        let priority = self.priority(graph, &shortcuts_results, vertex);

        (priority, shortcuts_results.shortcuts)
    }

    fn initialize(&mut self, graph: &Graph) {
        let mut vertices: Vec<u32> = (0..graph.number_of_vertices()).map(|x| x as u32).collect();
        vertices.shuffle(&mut rand::thread_rng());

        self.queue = vertices
            .into_par_iter()
            .progress()
            .map(|vertex| {
                let (priority, _) = self.priority_and_shortcuts(vertex, graph);
                CHState { vertex, priority }
            })
            .collect();
    }

    fn priority(
        &self,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
        vertex: u32,
    ) -> i32 {
        let priorities: Vec<i32> = self
            .priority_terms
            .iter()
            .map(|(coefficent, priority_function)| {
                coefficent * priority_function.priority(vertex, graph, shortcuts_results)
            })
            .collect();

        priorities.iter().sum::<i32>()
    }
}
