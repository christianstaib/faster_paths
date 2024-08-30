use std::collections::HashMap;

use rayon::prelude::*;

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Graph, TaillessEdge, Vertex, WeightedEdge},
    search::{
        ch::{contracted_graph::ContractedGraph, contraction_generic::contraction_bottom_up},
        DistanceHeuristic,
    },
};

impl ContractedGraph {
    pub fn by_contraction_with_heuristic<G: Graph + Default + Clone>(
        graph: &ReversibleGraph<G>,
        heuristic: &dyn DistanceHeuristic,
    ) -> ContractedGraph {
        let graph = graph.clone();
        let (level_to_vertex, edges, shortcuts) = contraction_bottom_up(graph, |graph, vertex| {
            par_simulate_contraction_heuristic(graph, heuristic, vertex)
        });

        ContractedGraph::new(level_to_vertex, edges, shortcuts)
    }
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_heuristic<G: Graph>(
    graph: &ReversibleGraph<G>,
    heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)> {
    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .par_bridge()
        .map(|in_edge| {
            let tail = in_edge.head;

            let mut new_edges = Vec::new();
            let mut updated_edges = Vec::new();

            for out_edge in graph.out_graph().edges(vertex) {
                let head = out_edge.head;

                if tail == head {
                    continue;
                }

                let shortcut_distance = in_edge.weight + out_edge.weight;
                let edge = WeightedEdge {
                    tail,
                    head,
                    weight: shortcut_distance,
                };

                // Checking current edge weight is propabally cheaper than heuristic so check
                // first
                if let Some(current_edge_weight) =
                    graph.out_graph().get_weight(&edge.remove_weight())
                {
                    if shortcut_distance < current_edge_weight {
                        updated_edges.push(edge.remove_tail());
                    }
                    continue;
                }

                if heuristic.is_less_or_equal_upper_bound(tail, head, shortcut_distance) {
                    new_edges.push(edge.remove_tail());
                }
            }

            (tail, (new_edges, updated_edges))
        })
        .collect()
}
