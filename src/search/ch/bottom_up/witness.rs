use std::collections::HashMap;

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, Distance, Graph, TaillessEdge, Vertex, WeightedEdge,
    },
    search::{
        ch::{
            contracted_graph::{new, ContractedGraph},
            contraction_generic::contraction_bottom_up,
        },
        collections::dijkstra_data::DijkstraData,
        dijkstra::dijkstra_one_to_many,
    },
};

impl ContractedGraph {
    pub fn with_dijkstra_witness_search<G: Graph + Default + Clone>(
        graph: &ReversibleGraph<G>,
        hop_limit: u32,
    ) -> ContractedGraph {
        let graph = graph.clone();
        let (level_to_vertex, edges, shortcuts) = contraction_bottom_up(graph, |graph, vertex| {
            par_simulate_contraction_witness_search(graph, hop_limit, vertex)
        });

        new(level_to_vertex, edges, shortcuts)
    }
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_witness_search<G: Graph>(
    graph: &ReversibleGraph<G>,
    hop_limit: u32,
    vertex: Vertex,
) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)> {
    // create vec of out neighbors once and reuse it afterwards
    let out_neighbors = graph.out_graph().neighbors(vertex);

    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .par_bridge()
        .map(|in_edge| {
            let tail = in_edge.head;

            let mut new_edges = Vec::new();
            let mut updated_edges = Vec::new();

            // Get all shortest path distances (tail -> neighbor)
            let data = dijkstra_one_to_many(graph.out_graph(), hop_limit, tail, &out_neighbors);

            for out_edge in graph.out_graph().edges(vertex) {
                let head = out_edge.head;

                if tail == head {
                    continue;
                }

                let shortcut_distance = in_edge.weight + out_edge.weight;
                let shortest_path_distance = data.get_distance(head).unwrap_or(Distance::MAX);

                if shortcut_distance <= shortest_path_distance {
                    let edge = WeightedEdge {
                        tail,
                        head,
                        weight: shortcut_distance,
                    };
                    if let Some(current_edge_weight) =
                        graph.out_graph().get_weight(&edge.remove_weight())
                    {
                        if shortcut_distance < current_edge_weight {
                            updated_edges.push(edge.remove_tail());
                        }
                    } else {
                        new_edges.push(edge.remove_tail());
                    }
                }
            }

            (tail, (new_edges, updated_edges))
        })
        .collect()
}
