use std::collections::BTreeSet;

use indicatif::ProgressBar;
use rayon::prelude::*;

use crate::{
    ch::Shortcut,
    graphs::{graph::Graph, types::VertexId},
};

use super::{contraction_helper::get_shortcuts, Contractor};

pub struct ParallelContractor {}

impl Contractor for ParallelContractor {
    fn contract(&self, graph: &Graph) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        let mut graph = graph.clone();
        let mut shortcuts = Vec::new();
        let bar = ProgressBar::new(graph.number_of_vertices() as u64);
        let mut remaining: BTreeSet<VertexId> = (0..graph.number_of_vertices()).collect();

        let mut levels = Vec::new();
        while !remaining.is_empty() {
            let ids = self.get_independen_set(&graph, &mut remaining);

            let mut this_level = Vec::new();
            for (v, mut this_shortcuts) in ids.into_iter() {
                graph.remove_vertex(v);
                this_level.push(v);
                this_shortcuts
                    .iter()
                    .for_each(|shortcut| graph.add_edge(&shortcut.edge));
                shortcuts.append(&mut this_shortcuts);
                bar.inc(1);
            }
            levels.push(this_level);
        }
        bar.finish();

        (shortcuts, levels)
    }
}

impl ParallelContractor {
    pub fn new() -> Self {
        ParallelContractor {}
    }

    pub fn get_independen_set(
        &self,
        graph: &Graph,
        vertices: &mut BTreeSet<u32>,
    ) -> Vec<(VertexId, Vec<Shortcut>)> {
        let mut ids = Vec::new();
        let mut neighbors = BTreeSet::new();

        for &vertex in vertices.iter() {
            if neighbors.contains(&vertex) {
                continue;
            }

            let vertex_neighborhood = graph.open_neighborhood(vertex, 1);
            neighbors.extend(vertex_neighborhood);

            ids.push(vertex);
        }

        ids.sort_by_cached_key(|&vertex| {
            graph.out_edges(vertex).len() * graph.in_edges(vertex).len()
        });

        ids.truncate(ids.len().div_ceil(2));

        for vertex in ids.iter() {
            vertices.remove(vertex);
        }

        let vertex_shortcuts: Vec<_> = ids
            .par_iter()
            .map(|&vertex| (vertex, get_shortcuts(&graph, vertex, 100)))
            .collect();

        vertex_shortcuts
            .into_iter()
            .map(|(vertex, results)| (vertex, results.shortcuts))
            .collect()
    }
}
