use std::{cmp::Reverse, collections::BinaryHeap};

use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::{
    contraction_hierachy::{
        ContractionEdge,
        contraction::{
            general::generate_shortcuts,
            terms::{Term, default_terms, priority},
        },
    },
    graph::{DirectionalAdjacencyListGraph, GraphLike},
    types::{Distance, VertexId},
};

const MAX_WITNESS_HOPS: u32 = 100;

pub(super) struct Queue<D: Distance> {
    heap: BinaryHeap<(Reverse<i64>, VertexId)>,
    terms: Vec<Box<dyn Term<D>>>,
}

impl<D: Distance + Send + Sync> Queue<D> {
    pub(super) fn new(graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>) -> Self {
        let terms = default_terms::<D>(graph.num_vertices());
        let heap = initial_heap(graph, &terms);
        Self { heap, terms }
    }

    pub(super) fn pop(
        &mut self,
        graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    ) -> Option<(VertexId, Vec<ContractionEdge<D>>)> {
        while let Some((Reverse(queued_priority), vertex)) = self.heap.pop() {
            let shortcuts = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS);
            let current_priority = priority(graph, vertex, &shortcuts, &self.terms);

            if current_priority <= queued_priority {
                self.update_terms_for_contracted(graph, vertex, &shortcuts);
                return Some((vertex, shortcuts));
            }

            self.heap.push((Reverse(current_priority), vertex));
        }

        None
    }

    fn update_terms_for_contracted(
        &mut self,
        graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    ) {
        for term in &mut self.terms {
            term.update(graph, vertex, shortcuts);
        }
    }
}

fn initial_heap<D: Distance + Send + Sync>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    terms: &[Box<dyn Term<D>>],
) -> BinaryHeap<(Reverse<i64>, VertexId)> {
    (0..graph.num_vertices() as u32)
        .into_par_iter()
        .progress()
        .map(VertexId::new)
        .map(|vertex| {
            let shortcuts = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS);
            (Reverse(priority(graph, vertex, &shortcuts, terms)), vertex)
        })
        .collect()
}
