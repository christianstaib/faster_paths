use std::sync::{Arc, Mutex};

use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Edge, Graph, Vertex, WeightedEdge},
    search::{
        self,
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataHashMap},
            vertex_distance_queue::VertexDistanceQueueBinaryHeap,
            vertex_expanded_data::VertexExpandedDataHashSet,
        },
        dijkstra::{dijkstra_one_to_many, dijktra_one_to_many},
        DistanceHeuristic,
    },
};

/// Simulates a contraction. Returns (new_edges, updated_edges)
pub fn probabilistic_edge_difference_witness_search<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    vertex: Vertex,
    min_searches: u32,
    max_searches: u32,
    search_factor: f32,
) -> i32 {
    let in_edges = graph.in_graph().edges(vertex).collect_vec();
    let mut searches = (in_edges.len() as f32 * search_factor) as u32;
    if searches < min_searches {
        searches = min_searches;
    } else if searches > max_searches {
        searches = max_searches;
    }
    let in_edges_selected = in_edges.choose_multiple(&mut thread_rng(), searches as usize);
    let selcted_factor = in_edges_selected.len() as f32 / in_edges.len() as f32;

    let out_neighbors = graph
        .out_graph()
        .edges(vertex)
        .map(|edge| edge.head)
        .collect_vec();

    let mut new_edges_len = 0;

    // tail -> vertex -> head
    in_edges_selected.for_each(|in_edge| {
        let tail = in_edge.head;

        // dijkstra tail -> targets
        let mut data = DijkstraDataHashMap::new();
        let mut expanded = VertexExpandedDataHashSet::new();
        let mut queue = VertexDistanceQueueBinaryHeap::new();
        dijktra_one_to_many(
            graph.out_graph(),
            &mut data,
            &mut expanded,
            &mut queue,
            tail,
            &out_neighbors,
        );

        graph.out_graph().edges(vertex).for_each(|out_edge| {
            let head = out_edge.head;
            let shortcut_distance = in_edge.weight + out_edge.weight;

            let shortest_path_distance = data.get_distance(head).unwrap_or(Distance::MAX);

            if shortcut_distance <= shortest_path_distance {
                let edge = WeightedEdge {
                    tail,
                    head,
                    weight: shortcut_distance,
                };
                if graph.get_weight(&edge.remove_weight()).is_some() {
                    // updated_edges.push(edge);
                } else {
                    // new_edges.push(edge);
                    new_edges_len += 1;
                }
            }
        })
    });

    let new_edges_len = (new_edges_len as f32 / selcted_factor) as i32;

    new_edges_len
        - graph.in_graph().edges(vertex).len() as i32
        - graph.out_graph().edges(vertex).len() as i32
}

/// Simulates a contraction. Returns (new_edges, updated_edges)
pub fn simulate_contraction_witness_search<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    vertex: Vertex,
) -> (Vec<WeightedEdge>, Vec<WeightedEdge>) {
    let out_neighbors = graph
        .out_graph()
        .edges(vertex)
        .map(|edge| edge.head)
        .collect_vec();

    let mut new_edges = Vec::new();
    let mut updated_edges = Vec::new();

    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .progress()
        .for_each(|in_edge| {
            let tail = in_edge.head;

            // dijkstra tail -> targets
            let data = dijkstra_one_to_many(graph.out_graph(), tail, &out_neighbors);

            graph.out_graph().edges(vertex).for_each(|out_edge| {
                let head = out_edge.head;
                let shortcut_distance = in_edge.weight + out_edge.weight;

                let shortest_path_distance = data.get_distance(head).unwrap_or(Distance::MAX);

                if shortcut_distance <= shortest_path_distance {
                    let edge = WeightedEdge {
                        tail,
                        head,
                        weight: shortcut_distance,
                    };
                    if graph.get_weight(&edge.remove_weight()).is_some() {
                        updated_edges.push(edge);
                    } else {
                        new_edges.push(edge);
                    }
                }
            })
        });

    (new_edges, updated_edges)
}

/// Simulates a contraction. Returns (new_edges, updated_edges)
pub fn par_simulate_contraction_witness_search<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    vertex: Vertex,
) -> (Vec<WeightedEdge>, Vec<WeightedEdge>) {
    let out_neighbors = graph
        .out_graph()
        .edges(vertex)
        .map(|edge| edge.head)
        .collect_vec();

    let new_edges = Arc::new(Mutex::new(Vec::new()));
    let updated_edges = Arc::new(Mutex::new(Vec::new()));

    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .progress()
        .par_bridge()
        .for_each(|in_edge| {
            let tail = in_edge.head;

            // dijkstra tail -> targets
            let data = dijkstra_one_to_many(graph.out_graph(), tail, &out_neighbors);

            graph.out_graph().edges(vertex).for_each(|out_edge| {
                let head = out_edge.head;
                let shortcut_distance = in_edge.weight + out_edge.weight;

                let shortest_path_distance = data.get_distance(head).unwrap_or(Distance::MAX);

                if shortcut_distance <= shortest_path_distance {
                    let edge = WeightedEdge {
                        tail,
                        head,
                        weight: shortcut_distance,
                    };
                    if graph.get_weight(&edge.remove_weight()).is_some() {
                        updated_edges.lock().unwrap().push(edge);
                    } else {
                        new_edges.lock().unwrap().push(edge);
                    }
                }
            })
        });

    (
        Arc::into_inner(new_edges).unwrap().into_inner().unwrap(),
        Arc::into_inner(updated_edges)
            .unwrap()
            .into_inner()
            .unwrap(),
    )
}

pub fn edge_difference<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    new_edges: &Vec<WeightedEdge>,
    vertex: Vertex,
) -> i32 {
    new_edges.len() as i32
        - graph.in_graph().edges(vertex).len() as i32
        - graph.out_graph().edges(vertex).len() as i32
}

pub fn simulate_contraction_distance_heuristic<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    distance_heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> (Vec<WeightedEdge>, Vec<WeightedEdge>) {
    let mut new_edges = Vec::new();
    let mut updated_edges = Vec::new();

    // tail -> vertex -> head
    graph.in_graph().edges(vertex).for_each(|in_edge| {
        let tail = in_edge.head;

        graph.out_graph().edges(vertex).for_each(|out_edge| {
            let head = out_edge.head;
            let shortcut_distance = in_edge.weight + out_edge.weight;

            let lower_bound_distance = distance_heuristic
                .lower_bound(tail, head)
                .unwrap_or(Distance::MAX);

            if shortcut_distance <= lower_bound_distance {
                let edge = WeightedEdge {
                    tail,
                    head,
                    weight: shortcut_distance,
                };
                if graph.get_weight(&edge.remove_weight()).is_some() {
                    updated_edges.push(edge);
                } else {
                    new_edges.push(edge);
                }
            }
        })
    });

    (new_edges, updated_edges)
}

pub fn probabilistic_edge_difference_distance_neuristic<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    distance_heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
    min_searches: u32,
    max_searches: u32,
    search_factor: f32,
) -> i32 {
    let in_edges = graph.in_graph().edges(vertex).collect_vec();
    let out_edges = graph.out_graph().edges(vertex).collect_vec();

    let number_of_edge_pairs = (in_edges.len() * out_edges.len()) as u32;

    if number_of_edge_pairs == 0 {
        return -(in_edges.len() as i32) - (out_edges.len() as i32);
    }

    let mut searches = (number_of_edge_pairs as f32 * search_factor) as u32;
    if searches < min_searches {
        searches = min_searches;
    } else if searches > max_searches {
        searches = max_searches;
    }

    if searches > number_of_edge_pairs {
        searches = number_of_edge_pairs;
    }
    let searches_factor = searches as f32 / number_of_edge_pairs as f32;

    let mut rng = thread_rng();
    let mut new_edges_len = 0;
    for _ in 0..searches {
        let in_edge = in_edges.choose(&mut rng).unwrap();
        let out_edge = in_edges.choose(&mut rng).unwrap();

        let shortcut_distance = in_edge.weight + out_edge.weight;
        let lower_bound = distance_heuristic
            .lower_bound(in_edge.head, out_edge.head)
            .unwrap_or(0);

        if shortcut_distance <= lower_bound {
            let edge = Edge {
                tail: in_edge.head,
                head: out_edge.head,
            };
            if graph.get_weight(&edge).is_none() {
                new_edges_len += 1;
            }
        }
    }

    let new_edges_len = ((new_edges_len as f32) / searches_factor) as i32;

    new_edges_len - in_edges.len() as i32 - out_edges.len() as i32
}
