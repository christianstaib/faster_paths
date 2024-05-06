use std::{collections::BinaryHeap, time::Instant};

use ahash::{HashMap, HashMapExt};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::prelude::SliceRandom;
use rayon::prelude::*;

use super::{contraction_helper::ShortcutGenerator, Contractor};
use crate::{
    ch::{ch_priority_element::ChPriorityElement, priority_function::PriorityFunction, Shortcut},
    graphs::{Graph, VertexId},
};

pub struct SerialWitnessSearchContractor {
    queue: BinaryHeap<ChPriorityElement>,
    priority_terms: Vec<(i32, Box<dyn PriorityFunction + Sync>)>,
    shortcut_generator: Box<dyn ShortcutGenerator>,
}

impl Contractor for SerialWitnessSearchContractor {
    /// Generates contraction hierarchy where one vertex at a time is
    /// contracted.
    fn contract(&mut self, mut graph: Box<dyn Graph>) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        println!("initalizing queue");
        self.initialize(&*graph);

        let mut shortcuts: HashMap<(VertexId, VertexId), Shortcut> = HashMap::new();
        let mut levels = Vec::new();

        // let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
        // writeln!(
        //     writer,
        //     "pop(ns),add_edge(ns),add_shortcuts(ns),remove_vertex(ns),num_edges,
        // num_shortcuts,num_new_shortcuts" )
        // .unwrap();

        println!("start contracting");
        let bar = ProgressBar::new(graph.number_of_vertices() as u64);

        let mut start = Instant::now();
        while let Some((vertex, vertex_shortcuts)) = self.pop(&*graph) {
            let _duration_pop = start.elapsed();

            // let duration_add_edge = start.elapsed();
            let _vertex_shortcut_len = vertex_shortcuts.len();

            // // check parallel which shortcuts to insert
            // let shortcuts_to_add_to_shortcuts: Vec<_> = vertex_shortcuts
            //     .into_par_iter()
            //     // only add shortcuts that are either not present or weigh less than a
            // current     // shortcut
            //     .filter(|shortcut| {
            //         let this_key = (
            //             shortcut.edge.unweighted().tail,
            //             shortcut.edge.unweighted().head,
            //         );
            //         if let Some(current_shortcut) = shortcuts.get(&this_key) {
            //             if shortcut.edge.weight >= current_shortcut.edge.weight {
            //                 return false;
            //             }
            //         }
            //         true
            //     })
            //     .collect();

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

            // println!(
            //     "{:>9}  {:>9} {:>9} {:>9} {:?}",
            //     vertex_shortcut_len,
            //     // shortcuts_to_add_to_shortcuts.len(),
            //     shortcuts_to_add_to_graph.len(),
            //     shortcuts.len(),
            //     graph.number_of_edges(),
            //     duration_pop
            // );

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

            // let duration_add_shortcuts = start.elapsed();

            graph.remove_vertex(vertex);

            // let duration_remove_vertex = start.elapsed();

            // writeln!(
            //     writer,
            //     "{},{},{},{},{},{},{}",
            //     duration_pop.as_nanos(),
            //     duration_add_edge.as_nanos() - duration_pop.as_nanos(),
            //     duration_add_shortcuts.as_nanos() - duration_add_edge.as_nanos(),
            //     duration_remove_vertex.as_nanos() - duration_add_shortcuts.as_nanos(),
            //     number_of_edges(&*graph),
            //     shortcuts.len(),
            //     vertex_shortcut_len
            // )
            // .unwrap();

            levels.push(vec![vertex]);
            bar.inc(1);
            start = Instant::now();
        }
        bar.finish();

        // writer.flush().unwrap();

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
