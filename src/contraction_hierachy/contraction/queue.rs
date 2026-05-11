use std::{cmp::Reverse, collections::BinaryHeap};

use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::{
    contraction_hierachy::{
        ContractionEdge,
        contraction::{
            general::{edge_difference, generate_shortcuts},
            working_graph::WorkingGraph,
        },
    },
    types::{Distance, VertexId},
};

const MAX_WITNESS_HOPS: u32 = 10;

pub(super) struct Queue {
    heap: BinaryHeap<(Reverse<i64>, VertexId)>,
}

impl Queue {
    pub(super) fn new<D: Distance + Sync>(graph: &WorkingGraph<D>) -> Self {
        let heap = initial_heap(graph);
        Self { heap }
    }

    pub(super) fn len(&self) -> usize {
        self.heap.len()
    }

    pub(super) fn pop<D: Distance>(
        &mut self,
        graph: &WorkingGraph<D>,
    ) -> Option<(VertexId, Vec<ContractionEdge<D>>)> {
        while let Some((Reverse(queued_edge_difference), vertex)) = self.heap.pop() {
            let shortcuts = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS);
            let current_edge_difference = edge_difference(graph, vertex, shortcuts.len());

            if current_edge_difference <= queued_edge_difference {
                return Some((vertex, shortcuts));
            }

            self.heap.push((Reverse(current_edge_difference), vertex));
        }

        None
    }
}

fn initial_heap<D: Distance + Sync>(
    graph: &WorkingGraph<D>,
) -> BinaryHeap<(Reverse<i64>, VertexId)> {
    (0..graph.num_vertices() as u32)
        .into_par_iter()
        .progress()
        .map(VertexId::new)
        .map(|vertex| {
            let shortcut_count = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS).len();
            (
                Reverse(edge_difference(graph, vertex, shortcut_count)),
                vertex,
            )
        })
        .collect()
}
