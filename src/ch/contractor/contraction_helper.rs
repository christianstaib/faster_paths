use itertools::Itertools;
use rayon::prelude::*;

use super::Shortcut;
use crate::{
    ch::contractor::witness_search::witness_search,
    graphs::{
        adjacency_vec_graph::AdjacencyVecGraph, edge::DirectedWeightedEdge,
        path::ShortestPathRequest, Graph, VertexId,
    },
    heuristics::Heuristic,
};

pub trait ShortcutGenerator: Send + Sync {
    fn get_shortcuts(&self, graph: &dyn Graph, vertex: VertexId) -> Vec<Shortcut>;
}

pub struct ShortcutGeneratorWithWittnessSearch {
    pub max_hops: u32,
}

impl ShortcutGenerator for ShortcutGeneratorWithWittnessSearch {
    fn get_shortcuts(&self, graph: &dyn Graph, vertex: VertexId) -> Vec<Shortcut> {
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
    fn get_shortcuts(&self, graph: &dyn Graph, vertex: VertexId) -> Vec<Shortcut> {
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
