use itertools::Itertools;
use rand::prelude::*;

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex, WeightedEdge},
    search::{
        self,
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataHashMap},
            vertex_distance_queue::VertexDistanceQueueBinaryHeap,
            vertex_expanded_data::VertexExpandedDataHashSet,
        },
        dijkstra::dijktra_one_to_many,
    },
};

/// Simulates a contraction. Returns (new_edges, updated_edges)
pub fn probabilistic_edge_difference<G: Graph + Default>(
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

    let mut new_edges = Vec::new();
    let mut updated_edges = Vec::new();

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
                    updated_edges.push(edge);
                } else {
                    new_edges.push(edge);
                }
            }
        })
    });

    let new_edges_len = (new_edges.len() as f32 / selcted_factor) as i32;

    new_edges_len
        - graph.in_graph().edges(vertex).len() as i32
        - graph.out_graph().edges(vertex).len() as i32
}

/// Simulates a contraction. Returns (new_edges, updated_edges)
pub fn simulate_contraction<G: Graph + Default>(
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
    graph.in_graph().edges(vertex).for_each(|in_edge| {
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
                    updated_edges.push(edge);
                } else {
                    new_edges.push(edge);
                }
            }
        })
    });

    (new_edges, updated_edges)
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
