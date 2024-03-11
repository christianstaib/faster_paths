use core::panic;
use rand::seq::SliceRandom;
use std::{cmp::Ordering, collections::BTreeSet, time::Instant, usize};

use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedEdge, graph::Graph, types::VertexId};

use super::{ch_queue::queue::CHQueue, contraction_helper::ContractionHelper, shortcut::Shortcut};

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub graph: Graph,
    pub shortcuts_map: Vec<(DirectedEdge, VertexId)>,
    pub levels: Vec<Vec<u32>>,
}

pub struct Contractor {
    graph: Graph,
    queue: CHQueue,
    levels: Vec<u32>,
}

impl Contractor {
    pub fn new(graph: &Graph, priority_functions: &str) -> Self {
        let levels = vec![0; graph.number_of_vertices() as usize];
        let graph = graph.clone();
        let queue = CHQueue::new(&graph, priority_functions);

        Contractor {
            graph,
            queue,
            levels,
        }
    }

    pub fn get_graph(mut self) -> ContractedGraph {
        let old_graph = self.graph.clone();

        let shortcuts = self.contract_single_vertex();

        self.graph = old_graph;
        self.add_shortcuts_to_graph(&shortcuts);
        self.removing_edges_violating_level_property();

        let shortcuts = shortcuts
            .iter()
            .map(|shortcut| (shortcut.edge.unweighted(), shortcut.skiped_vertex))
            .collect();

        let max_level = self.levels.iter().max().unwrap();
        let mut levels = vec![Vec::new(); *max_level as usize + 1];

        for (vertex, level) in self.levels.iter().enumerate() {
            levels[*level as usize].push(vertex as u32);
        }

        ContractedGraph {
            graph: self.graph,
            shortcuts_map: shortcuts,
            levels,
        }
    }

    /// Generates contraction hierarchy where one vertex at a time is contracted.
    pub fn contract_single_vertex(&mut self) -> Vec<Shortcut> {
        let mut shortcuts = Vec::new();

        let bar = ProgressBar::new(self.graph.number_of_vertices() as u64);

        let mut level = 0;
        while let Some(v) = self.queue.pop(&self.graph) {
            let mut this_shortcuts = v.1;
            let v = v.0;

            self.add_shortcuts_to_graph(&this_shortcuts);
            shortcuts.append(&mut this_shortcuts);

            self.graph.remove_vertex(v);
            self.levels[v as usize] = level;

            level += 1;
            bar.inc(1);
        }
        bar.finish();

        shortcuts
    }

    pub fn create_independen_set(
        &self,
        vertices: &BTreeSet<u32>,
        priority: &Vec<i32>,
        k: u32,
    ) -> Vec<VertexId> {
        let ids: Vec<_> = vertices
            .par_iter()
            .filter(|&&vertex| {
                // let neighbors = self.graph.open_neighborhood(vertex, k);
                let neighbors = self.graph.open_neighborhood_dijkstra(vertex, k);
                // assert_eq!(neighbors.len(), n2.len());
                for &neighbor in neighbors.iter() {
                    // break if there is a neighbor who is less important
                    if priority[neighbor as usize]
                        .cmp(&priority[vertex as usize])
                        .then_with(|| neighbor.cmp(&vertex))
                        == Ordering::Less
                    {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        ids
    }

    pub fn get_independen_set(
        &self,
        remaining: &mut BTreeSet<VertexId>,
        priority: &Vec<i32>,
        k: u32,
    ) -> Vec<VertexId> {
        let ids = self.create_independen_set(remaining, priority, k);
        for vertex in ids.iter() {
            remaining.remove(vertex);
        }
        ids
    }

    pub fn get_necessary_shortcuts(
        &self,
        ids: &Vec<VertexId>,
        shortcuts_cache: &mut Vec<Vec<Shortcut>>,
    ) -> Vec<Shortcut> {
        let shortcut_generator = ContractionHelper::new(&self.graph, 100, u32::MAX);
        ids.par_iter()
            //.map(|&vertex| std::mem::take(&mut shortcuts_cache[vertex as usize]))
            .map(|&vertex| shortcut_generator.get_shortcuts(vertex).shortcuts)
            .flatten()
            .collect()
    }

    pub fn contract_ids(&mut self) -> Vec<Shortcut> {
        let mut shortcuts = Vec::new();
        let bar = ProgressBar::new(self.graph.number_of_vertices() as u64);
        let mut remaining: BTreeSet<VertexId> = (0..self.graph.number_of_vertices()).collect();

        let mut priority = vec![0; self.graph.number_of_vertices() as usize];
        let mut shortcuts_cache = vec![Vec::new(); self.graph.number_of_vertices() as usize];

        let mut level = 0;

        self.update_vertices(&remaining, &mut priority, &mut shortcuts_cache);

        while !remaining.is_empty() {
            println!("");
            let start = Instant::now();
            //  I <- Independent Node Set
            let ids = self.get_independen_set(&mut remaining, &priority, 2);
            println!("getting ids took {:?}", start.elapsed());
            // needs to be done before as afterwards neighbor art not known anymore
            let start = Instant::now();
            let ids_neighbors: BTreeSet<_> = ids
                .par_iter()
                .map(|&vertex| {
                    let n1 = self.graph.open_neighborhood_dijkstra(vertex, 1);
                    // let n2 = self.graph.open_neighborhood_dijkstra(vertex, 1);
                    // assert!(n1.len() == n2.len());
                    n1
                })
                .flatten()
                .collect();
            println!("getting neighbors took {:?}", start.elapsed());

            assert!(!ids.is_empty());

            let start = Instant::now();
            // E <- Necessary Shortcuts
            let mut ids_shortcuts = self.get_necessary_shortcuts(&ids, &mut shortcuts_cache);
            println!("getting shortcuts took {:?}", start.elapsed());

            let start = Instant::now();
            // Move I to their Level
            for &v in ids.iter() {
                self.graph.remove_vertex(v);
                self.levels[v as usize] = level;
            }

            // Insert E into Remaining graph
            self.add_shortcuts_to_graph(&ids_shortcuts);
            shortcuts.append(&mut ids_shortcuts);
            println!("graph stuff took {:?}", start.elapsed());

            let start = Instant::now();
            // Update Priority of Neighbors of I with Simulated Contractions
            self.update_vertices(&ids_neighbors, &mut priority, &mut shortcuts_cache);
            println!("updating neighbors took {:?}", start.elapsed());

            bar.inc(ids.len() as u64);
            level += 1;
        }
        bar.finish();

        shortcuts
    }

    fn update_vertices(
        &mut self,
        vertices: &BTreeSet<u32>,
        priority: &mut Vec<i32>,
        shortcuts_cache: &mut Vec<Vec<Shortcut>>,
    ) {
        let shortcut_generator = ContractionHelper::new(&self.graph, 100, u32::MAX);
        let v_shortcuts_difference: Vec<_> = vertices
            .par_iter()
            .map(|&vertex| {
                let results = shortcut_generator.get_shortcuts(vertex);
                (vertex, results.shortcuts, results.edge_difference)
            })
            .collect();
        for (vertex, shortcuts, edge_difference) in v_shortcuts_difference.into_iter() {
            priority[vertex as usize] = edge_difference;
            shortcuts_cache[vertex as usize] = shortcuts;
        }
    }

    fn add_shortcuts_to_graph(&mut self, shortcuts: &Vec<Shortcut>) {
        shortcuts
            .iter()
            .cloned()
            .for_each(|shortcut| self.graph.add_edge(&shortcut.edge));
    }

    fn removing_edges_violating_level_property(&mut self) {
        let num_nodes = self.graph.number_of_vertices();
        let mut out_edges: Vec<_> = (0..num_nodes)
            .map(|tail| self.graph.out_edges(tail).clone())
            .collect();
        let mut in_edges: Vec<_> = (0..num_nodes)
            .map(|tail| self.graph.in_edges(tail).clone())
            .collect();

        out_edges.iter_mut().enumerate().for_each(|(tail, edges)| {
            edges.retain(|edge| self.levels[edge.head as usize] >= self.levels[tail as usize]);
        });

        in_edges.iter_mut().enumerate().for_each(|(head, edges)| {
            edges.retain(|edge| self.levels[head as usize] <= self.levels[edge.tail as usize]);
        });

        self.graph = Graph::from_out_in_edges(out_edges, in_edges);
    }
}
