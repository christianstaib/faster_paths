use std::{collections::BTreeSet, time::Instant, usize};

use indicatif::ProgressBar;
use rayon::{prelude::*, result};
use serde_derive::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedEdge, graph::Graph, types::VertexId};

use super::{
    ch_queue::queue::CHQueue, contraction_helper::ContractionHelper, preprocessor::ContractedGraph,
    shortcut::Shortcut,
};

pub struct SerialContractor {
    graph: Graph,
    queue: CHQueue,
    levels: Vec<u32>,
}

impl SerialContractor {
    pub fn new(graph: &Graph, priority_functions: &str) -> Self {
        let levels = vec![0; graph.number_of_vertices() as usize];
        let graph = graph.clone();
        let queue = CHQueue::new(&graph, priority_functions);

        SerialContractor {
            graph,
            queue,
            levels,
        }
    }

    pub fn get_graph(mut self) -> ContractedGraph {
        let old_graph = self.graph.clone();

        let shortcuts = self.contract_ids();

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

    pub fn create_independen_set(
        &self,
        vertices: &BTreeSet<u32>,
        priority: &Vec<i32>,
        k: u32,
    ) -> Vec<(VertexId, Vec<Shortcut>)> {
        let mut ids = Vec::new();
        let mut neighbors = BTreeSet::new();

        for &vertex in vertices.iter() {
            if neighbors.contains(&vertex) {
                continue;
            }

            let vertex_neighborhood = self.graph.open_neighborhood(vertex, 1);
            neighbors.extend(vertex_neighborhood);

            ids.push(vertex);
        }

        ids.sort_by_cached_key(|&vertex| {
            self.graph.out_edges(vertex).len() * self.graph.in_edges(vertex).len()
        });

        ids.truncate(ids.len().div_ceil(2));

        let shortcut_generator = ContractionHelper::new(&self.graph, 100, u32::MAX);
        let mut vertex_shortcuts: Vec<_> = ids
            .par_iter()
            .map(|&vertex| (vertex, shortcut_generator.get_shortcuts(vertex)))
            .collect();
        // vertex_shortcuts.sort_unstable_by_key(|(_, results)| results.edge_difference);

        // vertex_shortcuts.truncate(vertex_shortcuts.len().div_ceil(2));

        vertex_shortcuts
            .into_iter()
            .map(|(vertex, results)| (vertex, results.shortcuts))
            .collect()
    }

    pub fn get_independen_set(
        &self,
        remaining: &mut BTreeSet<VertexId>,
        priority: &Vec<i32>,
        k: u32,
    ) -> Vec<(VertexId, Vec<Shortcut>)> {
        let ids = self.create_independen_set(remaining, priority, k);
        for (vertex, _) in ids.iter() {
            remaining.remove(vertex);
        }
        ids
    }

    pub fn contract_ids(&mut self) -> Vec<Shortcut> {
        let mut shortcuts = Vec::new();
        let bar = ProgressBar::new(self.graph.number_of_vertices() as u64);
        let mut remaining: BTreeSet<VertexId> = (0..self.graph.number_of_vertices()).collect();

        let mut priority = vec![0; self.graph.number_of_vertices() as usize];
        let mut shortcuts_cache = vec![Vec::new(); self.graph.number_of_vertices() as usize];

        let mut already_contracted = BTreeSet::new();
        let mut level = 0;

        self.update_vertices(&remaining, &mut priority, &mut shortcuts_cache);

        while !remaining.is_empty() {
            let ids = self.get_independen_set(&mut remaining, &priority, 1);

            ids.iter().for_each(|(vertex, _)| {
                already_contracted.insert(*vertex);
            });

            ids.iter()
                .map(|(_, shortuts)| shortuts.clone())
                .flatten()
                .for_each(|shortcut| {
                    assert!(!already_contracted.contains(&shortcut.edge.head));
                    assert!(!already_contracted.contains(&shortcut.edge.tail));
                });

            // Move I to their Level
            for (v, mut this_shortcuts) in ids.into_iter() {
                self.graph.remove_vertex(v);
                self.levels[v as usize] = level;
                self.add_shortcuts_to_graph(&this_shortcuts);
                shortcuts.append(&mut this_shortcuts);
                bar.inc(1);
            }

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
