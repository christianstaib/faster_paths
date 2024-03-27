use std::collections::BinaryHeap;

use indicatif::ParallelProgressIterator;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    ch::{contractor::contraction_helper::get_shortcuts, Shortcut},
    graphs::{graph::Graph, VertexId},
};

use super::{
    ch_priority_element::ChPriorityElement,
    priority_function::{
        cost_of_queries::CostOfQueries, deleted_neighbors::DeletedNeighbors,
        edge_difference::EdgeDifference, search_space_size::SearchSpaceSize, PriorityFunction,
    },
};

pub struct CHQueue {
    queue: BinaryHeap<ChPriorityElement>,
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
        for letter in priority_functions_letters.split("_") {
            let letter = letter.split(":").collect::<Vec<_>>();
            let priority_function = *letter.get(0).unwrap();
            let coefficent = letter.get(1).unwrap().parse::<u32>().unwrap();
            match priority_function {
                "E" => queue.register(coefficent, EdgeDifference::new(graph)),
                "D" => queue.register(coefficent, DeletedNeighbors::new(graph)),
                "C" => queue.register(coefficent, CostOfQueries::new(graph)),
                "S" => queue.register(coefficent, SearchSpaceSize::new(graph)),
                _ => panic!("letter not recognized"),
            }
        }
        queue.initialize(graph);
        queue
    }

    fn register(
        &mut self,
        coefficent: u32,
        priority_function: impl PriorityFunction + 'static + Sync,
    ) {
        self.priority_terms
            .push((coefficent as i32, Box::new(priority_function)));
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
        let shortcuts_results = get_shortcuts(graph, vertex, 8);
        let priority = self
            .priority_terms
            .iter()
            .map(|(coefficent, priority_function)| {
                coefficent * priority_function.priority(vertex, graph, &shortcuts_results)
            })
            .sum();

        (priority, shortcuts_results.shortcuts)
    }

    fn initialize(&mut self, graph: &Graph) {
        let vertices: Vec<u32> = (0..graph.number_of_vertices()).collect();
        // vertices.shuffle(&mut rand::thread_rng());

        self.queue = vertices
            .into_par_iter()
            .progress()
            .map(|vertex| {
                let (priority, _) = self.priority_and_shortcuts(vertex, graph);
                ChPriorityElement { vertex, priority }
            })
            .collect();
    }
}
