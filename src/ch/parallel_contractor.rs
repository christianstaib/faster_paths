use std::collections::BTreeSet;

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

    pub fn contract(mut self) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        let mut shortcuts = Vec::new();
        let bar = ProgressBar::new(self.graph.number_of_vertices() as u64);
        let mut remaining: BTreeSet<VertexId> = (0..self.graph.number_of_vertices()).collect();

        let mut levels = Vec::new();
        while !remaining.is_empty() {
            let ids = self.get_independen_set(&mut remaining);

            let mut this_level = Vec::new();
            for (v, mut this_shortcuts) in ids.into_iter() {
                self.graph.remove_vertex(v);
                this_level.push(v);
                this_shortcuts
                    .iter()
                    .for_each(|shortcut| self.graph.add_edge(&shortcut.edge));
                shortcuts.append(&mut this_shortcuts);
                bar.inc(1);
            }
            levels.push(this_level);
        }
        bar.finish();

        (shortcuts, levels)
    }

    pub fn get_independen_set(
        &self,
        vertices: &mut BTreeSet<u32>,
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

        for vertex in ids.iter() {
            vertices.remove(vertex);
        }

        let shortcut_generator = ContractionHelper::new(&self.graph, 100);
        let vertex_shortcuts: Vec<_> = ids
            .par_iter()
            .map(|&vertex| (vertex, shortcut_generator.get_shortcuts(vertex)))
            .collect();

        vertex_shortcuts
            .into_iter()
            .map(|(vertex, results)| (vertex, results.shortcuts))
            .collect()
    }
}
