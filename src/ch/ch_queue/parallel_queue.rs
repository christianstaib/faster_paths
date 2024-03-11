use std::cmp::Ordering;

use ahash::HashSet;
use indicatif::ParallelProgressIterator;
use rand::seq::SliceRandom;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    ch::{
        contraction_helper::{ContractionHelper, ShortcutSearchResult},
        shortcut::Shortcut,
    },
    graphs::{graph::Graph, types::VertexId},
};

use super::{edge_difference::EdgeDifference, priority_function::PriorityFunction};

pub struct ParallelQueue {
    k: u32,
    remaining: Vec<VertexId>,
    order: Vec<i32>,
    shortcuts: Vec<Vec<Shortcut>>,
    priority_terms: Vec<(i32, Box<dyn PriorityFunction + Sync>)>,
}

impl ParallelQueue {
    pub fn pop(&mut self, graph: &Graph) -> Option<Vec<(VertexId, Vec<Shortcut>)>> {
        if self.remaining.is_empty() {
            return None;
        }

        let ids: Vec<_> = self
            .remaining
            .par_iter()
            .filter(|&&vertex| {
                let neighbors_with_higher_oder = graph
                    .open_neighborhood(vertex, self.k)
                    .iter()
                    .filter(|&&neighbor| {
                        self.order[neighbor as usize]
                            .cmp(&self.order[vertex as usize])
                            .then_with(|| neighbor.cmp(&vertex))
                            == Ordering::Greater
                    })
                    .count();
                neighbors_with_higher_oder == 0
            })
            .cloned()
            .collect();

        ids.iter().for_each(|&vertex| {
            let vertex_index = self.remaining.binary_search(&vertex).unwrap();
            self.remaining.remove(vertex_index);
        });

        if ids.is_empty() {
            panic!("should not happen as remaining is not empty");
        }

        Some(
            ids.iter()
                .map(|&vertex| (vertex, self.shortcuts.remove(vertex as usize)))
                .collect(),
        )
    }

    fn register(
        &mut self,
        coefficent: i32,
        priority_function: impl PriorityFunction + 'static + Sync,
    ) {
        self.priority_terms
            .push((coefficent, Box::new(priority_function)));
    }

    pub fn new(graph: &Graph, k: u32) -> Self {
        let remaining = (0..graph.number_of_vertices()).collect();
        let order = Vec::new();
        let shortcuts = Vec::new();
        let priority_terms = Vec::new();
        let mut queue = Self {
            k,
            remaining,
            order,
            shortcuts,
            priority_terms,
        };

        queue.register(1, EdgeDifference::new(&graph));

        let priority_and_shortcuts: Vec<_> = (0..graph.number_of_vertices())
            .into_par_iter()
            .map(|vertex| queue.get_priority_and_shortcuts(vertex, &graph))
            .collect();

        queue.order = priority_and_shortcuts
            .iter()
            .map(|(priority, _)| *priority)
            .collect();

        queue.shortcuts = priority_and_shortcuts
            .into_iter()
            .map(|(_, shortcuts)| shortcuts)
            .collect();

        queue
    }

    pub fn update(&mut self, verticies: &HashSet<VertexId>, graph: &Graph) {
        let priority_and_shortcuts: Vec<_> = verticies
            .into_par_iter()
            .map(|&vertex| self.get_priority_and_shortcuts(vertex, &graph))
            .collect();
        verticies
            .iter()
            .zip(priority_and_shortcuts.into_iter())
            .for_each(|(vertex, (priority, shortcuts))| {
                self.order[*vertex as usize] = priority;
                self.shortcuts[*vertex as usize] = shortcuts;
            });
    }

    pub fn get_priority_and_shortcuts(
        &self,
        vertex: VertexId,
        graph: &Graph,
    ) -> (i32, Vec<Shortcut>) {
        let shortcut_generator = ContractionHelper::new(graph, 100);
        let shortcuts_results = shortcut_generator.get_shortcuts(vertex);
        let priority = self
            .priority_terms
            .iter()
            .map(|(coefficent, priority_function)| {
                coefficent * priority_function.priority(vertex, graph, &shortcuts_results)
            })
            .sum();

        (priority, shortcuts_results.shortcuts)
    }
}
