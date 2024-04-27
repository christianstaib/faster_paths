use std::collections::HashMap;

use ahash::HashSet;
use itertools::Itertools;
use rayon::prelude::*;

use super::Shortcut;
use crate::{
    graphs::{edge::DirectedWeightedEdge, path::ShortestPathRequest, Graph, VertexId, Weight},
    heuristics::Heuristic,
    queue::{radix_queue::RadixQueue, DijkstaQueue, DijkstraQueueElement},
};

pub trait ShortcutGenerator: Send + Sync {
    fn get_shortcuts(&self, graph: &Box<dyn Graph>, vertex: VertexId) -> Vec<Shortcut>;
}

pub struct ShortcutGeneratorWithWittnessSearch {
    pub max_hops: u32,
}

impl ShortcutGenerator for ShortcutGeneratorWithWittnessSearch {
    fn get_shortcuts(&self, graph: &Box<dyn Graph>, vertex: VertexId) -> Vec<Shortcut> {
        let max_out_edge_weight = graph
            .in_edges(vertex)
            .map(|edge| edge.weight())
            .max()
            .unwrap_or(0);

        let heads = graph.out_edges(vertex).map(|edge| edge.head()).collect();

        graph
            .in_edges(vertex)
            .par_bridge()
            .flat_map(|in_edge| {
                let tail = in_edge.tail();
                let max_search_weight = in_edge.weight() + max_out_edge_weight;
                let witness_cost = witness_search(
                    graph,
                    tail,
                    vertex,
                    max_search_weight,
                    self.max_hops,
                    &heads,
                );

                graph
                    .out_edges(vertex)
                    .filter_map(|out_ede| {
                        let head = out_ede.head();
                        let weight = in_edge.weight() + out_ede.weight();

                        if &weight >= witness_cost.get(&head).unwrap_or(&u32::MAX) {
                            // (tail -> vertex -> head) is not THE shortest path from tail to head
                            return None;
                        }

                        let edge = DirectedWeightedEdge::new(tail, head, weight).unwrap();
                        Some(Shortcut { edge, vertex })
                    })
                    .collect_vec()
            })
            .collect()
    }
}

pub struct ShortcutGeneratorWithHeuristic {
    pub heuristic: Box<dyn Heuristic>,
}

impl ShortcutGenerator for ShortcutGeneratorWithHeuristic {
    fn get_shortcuts(&self, graph: &Box<dyn Graph>, vertex: VertexId) -> Vec<Shortcut> {
        graph
            .in_edges(vertex)
            .par_bridge()
            .flat_map(|in_edge| {
                let tail = in_edge.tail();
                graph
                    .out_edges(vertex)
                    .filter_map(|out_ede| {
                        let head = out_ede.head();
                        let weight = in_edge.weight() + out_ede.weight();

                        let request = ShortestPathRequest::new(in_edge.tail(), out_ede.head())?;
                        let upper_bound_uw_weight =
                            self.heuristic.upper_bound(&request).unwrap_or(u32::MAX);

                        if weight > upper_bound_uw_weight {
                            // (tail -> vertex -> head) is not A shortest path from tail to head
                            return None;
                        }

                        let edge = DirectedWeightedEdge::new(tail, head, weight).unwrap();
                        Some(Shortcut { edge, vertex })
                    })
                    .collect_vec()
            })
            .collect()
    }
}

pub fn witness_search(
    graph: &Box<dyn Graph>,
    source: VertexId,
    without: VertexId,
    max_weight: Weight,
    max_hops: u32,
    targets: &HashSet<VertexId>,
) -> HashMap<VertexId, Weight> {
    let mut queue = RadixQueue::new();
    let mut weight = HashMap::new();
    let mut hops = HashMap::new();

    let mut targets = targets.clone();

    queue.push(DijkstraQueueElement::new(0, source));
    weight.insert(source, 0);
    hops.insert(source, 0);

    while let Some(DijkstraQueueElement { vertex, .. }) = queue.pop() {
        if targets.remove(&vertex) && targets.is_empty() {
            break;
        }

        for edge in graph.out_edges(vertex) {
            let alternative_weight = weight[&vertex] + edge.weight();
            let alternative_hops = hops[&vertex] + 1;
            if (edge.head() != without)
                && (alternative_weight <= max_weight)
                && (alternative_hops <= max_hops)
            {
                let current_cost = *weight.get(&edge.head()).unwrap_or(&u32::MAX);
                if alternative_weight < current_cost {
                    queue.push(DijkstraQueueElement::new(alternative_weight, edge.head()));
                    weight.insert(edge.head(), alternative_weight);
                    hops.insert(edge.head(), alternative_hops);
                }
            }
        }
    }

    weight
}
