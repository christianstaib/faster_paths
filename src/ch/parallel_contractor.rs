use std::{collections::BTreeSet, usize};

use indicatif::ProgressBar;
use rayon::prelude::*;

use crate::graphs::{graph::Graph, types::VertexId};

use super::{contraction_helper::ContractionHelper, shortcut::Shortcut};

pub struct ParallelContractor {
    graph: Graph,
}

impl ParallelContractor {
    pub fn new(graph: &Graph) -> Self {
        let graph = graph.clone();

        ParallelContractor { graph }
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

    pub fn contract_ids(mut self) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        let mut shortcuts = Vec::new();
        let bar = ProgressBar::new(self.graph.number_of_vertices() as u64);
        let mut remaining: BTreeSet<VertexId> = (0..self.graph.number_of_vertices()).collect();

        let mut priority = vec![0; self.graph.number_of_vertices() as usize];
        let mut shortcuts_cache = vec![Vec::new(); self.graph.number_of_vertices() as usize];

        let mut already_contracted = BTreeSet::new();
        let mut level = 0;

        self.update_vertices(&remaining, &mut priority, &mut shortcuts_cache);

        let mut levels = Vec::new();
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
            let mut this_level = Vec::new();
            for (v, mut this_shortcuts) in ids.into_iter() {
                self.graph.remove_vertex(v);
                this_level.push(v);
                self.add_shortcuts_to_graph(&this_shortcuts);
                shortcuts.append(&mut this_shortcuts);
                bar.inc(1);
            }
            levels.push(this_level);

            level += 1;
        }
        bar.finish();

        (shortcuts, levels)
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
}
