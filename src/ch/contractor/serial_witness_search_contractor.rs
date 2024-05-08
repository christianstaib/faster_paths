use std::{collections::BinaryHeap, time::Instant};

use ahash::{HashMap, HashMapExt};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::prelude::SliceRandom;
use rayon::prelude::*;

use super::{contraction_helper::ShortcutGenerator, Contractor};
use crate::{
    ch::{ch_priority_element::ChPriorityElement, priority_function::PriorityFunction, Shortcut},
    graphs::{
        graph_functions::all_edges, reversible_vec_graph::ReversibleVecGraph, Graph, VertexId,
    },
};

pub struct SerialWitnessSearchContractor {
    queue: BinaryHeap<ChPriorityElement>,
    priority_terms: Vec<(i32, Box<dyn PriorityFunction + Sync>)>,
    shortcut_generator: Box<dyn ShortcutGenerator>,
}

impl Contractor for SerialWitnessSearchContractor {
    /// Generates contraction hierarchy where one vertex at a time is
    /// contracted.
    fn contract(&mut self, graph: &dyn Graph) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        let mut graph = Box::new(ReversibleVecGraph::from_edges(&all_edges(graph)));

        println!("initalizing queue");
        self.initialize(&*graph);

        let mut shortcuts: HashMap<(VertexId, VertexId), Shortcut> = HashMap::new();
        let mut levels = Vec::new();

        println!("start contracting");
        let bar = ProgressBar::new(graph.number_of_vertices() as u64);

        let mut start = Instant::now();
        while let Some((vertex, vertex_shortcuts)) = self.pop(&*graph) {
            let _duration_pop = start.elapsed();

            let _vertex_shortcut_len = vertex_shortcuts.len();

            let shortcuts_to_add_to_graph: Vec<_> = vertex_shortcuts
                .par_iter()
                .filter(|&shortcut| {
                    let current_weight = graph
                        .get_edge_weight(&shortcut.edge.unweighted())
                        .unwrap_or(u32::MAX);
                    shortcut.edge.weight() < current_weight
                })
                .cloned()
                .collect();

            shortcuts_to_add_to_graph.iter().for_each(|shortcut| {
                graph.set_edge(&shortcut.edge);
            });

            // insert serial
            for shortcut in shortcuts_to_add_to_graph {
                let this_key = (
                    shortcut.edge.unweighted().tail(),
                    shortcut.edge.unweighted().head(),
                );
                shortcuts.insert(this_key, shortcut);
            }

            graph.remove_vertex(vertex);

            levels.push(vec![vertex]);
            bar.inc(1);
            start = Instant::now();
        }
        bar.finish();

        (shortcuts.into_values().collect(), levels)
    }
}

impl SerialWitnessSearchContractor {
    // Lazy poping the vertex with minimum priority.
    pub fn pop(&mut self, graph: &dyn Graph) -> Option<(VertexId, Vec<Shortcut>)> {
        while let Some(mut state) = self.queue.pop() {
            // If current priority is greater than minimum priority, then repush state with
            // updated priority.
            let (priority, shortcuts) = self.priority_and_shortcuts(state.vertex, graph);
            if priority > state.priority {
                state.priority = priority;
                self.queue.push(state);
                continue;
            }

            // Gets called just before a vertex is contracted. Gives priority terms the
            // oppernunity to updated neighboring nodes priorities.
            self.priority_terms
                .iter_mut()
                .for_each(|(_, priority_function)| priority_function.update(state.vertex, &*graph));

            return Some((state.vertex, shortcuts));
        }
        None
    }

    pub fn priority_and_shortcuts(
        &self,
        vertex: VertexId,
        graph: &dyn Graph,
    ) -> (i32, Vec<Shortcut>) {
        let shortcuts = self.shortcut_generator.get_shortcuts(graph, vertex);
        let priority = self
            .priority_terms
            .iter()
            .map(|(coefficent, priority_function)| {
                coefficent * priority_function.priority(vertex, graph, &shortcuts)
            })
            .sum();

        (priority, shortcuts)
    }

    fn initialize(&mut self, graph: &dyn Graph) {
        let mut vertices: Vec<u32> = (0..graph.number_of_vertices()).collect();
        vertices.shuffle(&mut rand::thread_rng());

        self.priority_terms
            .iter_mut()
            .for_each(|(_, function)| function.initialize(graph));

        self.queue = vertices
            .into_par_iter()
            .progress()
            .map(|vertex| {
                let (priority, _) = self.priority_and_shortcuts(vertex, graph);
                ChPriorityElement { vertex, priority }
            })
            .collect();
    }

    pub fn new(
        priority_terms: Vec<(i32, Box<dyn PriorityFunction + Sync>)>,
        shortcut_generator: Box<dyn ShortcutGenerator>,
    ) -> Self {
        SerialWitnessSearchContractor {
            priority_terms,
            queue: BinaryHeap::new(),
            shortcut_generator,
        }
    }
}
