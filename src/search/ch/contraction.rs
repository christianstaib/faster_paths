use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, Distance, Graph, TaillessEdge, Vertex, WeightedEdge,
    },
    search::{collections::dijkstra_data::DijkstraData, dijkstra::dijkstra_one_to_many},
    utility::get_progressbar_long_jobs,
};

pub fn contraction_with_witness_search<G: Graph + Default>(
    mut graph: ReversibleGraph<G>,
    hop_limit: u32,
) -> (
    Vec<Vertex>,
    HashMap<(Vertex, Vertex), Distance>,
    HashMap<(Vertex, Vertex), Vertex>,
) {
    println!("setting up the queue");
    let mut queue = new_queue(&graph, hop_limit);

    println!("contracting");
    let mut edges = new_edge_map(&graph);
    let mut shortcuts = HashMap::new();

    let mut level_to_vertex = Vec::new();

    let pb =
        get_progressbar_long_jobs("Contracting", graph.out_graph().number_of_vertices() as u64);
    while let Some(Reverse((old_edge_difference, vertex))) = queue.pop() {
        let new_and_updated_edges =
            par_simulate_contraction_witness_search(&graph, hop_limit, vertex);
        let new_edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
        if new_edge_difference > old_edge_difference {
            queue.push(Reverse((new_edge_difference, vertex)));
            continue;
        }
        pb.inc(1);

        update_edge_map(&mut edges, &mut shortcuts, vertex, &new_and_updated_edges);

        level_to_vertex.push(vertex);
        graph.disconnect(vertex);
        graph.insert_and_update(&new_and_updated_edges);
    }
    pb.finish_and_clear();

    (level_to_vertex, edges, shortcuts)
}

pub fn update_edge_map(
    edge_map: &mut HashMap<(Vertex, Vertex), Distance>,
    shortcuts: &mut HashMap<(Vertex, Vertex), Vertex>,
    vertex: Vertex,
    new_and_updated_edges: &HashMap<u32, (Vec<TaillessEdge>, Vec<TaillessEdge>)>,
) {
    for (&tail, (new_edges, updated_edges)) in new_and_updated_edges.iter() {
        for edge in new_edges.iter().chain(updated_edges.iter()) {
            edge_map.insert((tail, edge.head), edge.weight);
            assert_ne!(tail, edge.head);
            assert_ne!(edge.head, vertex);
            assert_ne!(tail, vertex);
            shortcuts.insert((tail, edge.head), vertex);
        }
    }
}

pub fn new_edge_map<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
) -> HashMap<(Vertex, Vertex), Distance> {
    let mut edges = HashMap::new();
    for vertex in (0..graph.out_graph().number_of_vertices()).progress() {
        for edge in graph.out_graph().edges(vertex) {
            edges.insert((edge.tail, edge.head), edge.weight);
        }
    }
    edges
}

fn new_queue<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    hop_limit: u32,
) -> BinaryHeap<Reverse<(i32, u32)>> {
    graph
        .out_graph()
        .vertices()
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let new_and_updated_edges =
                par_simulate_contraction_witness_search(graph, hop_limit, vertex);
            let edge_difference = edge_difference(graph, &new_and_updated_edges, vertex);
            Reverse((edge_difference, vertex))
        })
        .collect()
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_witness_search<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    hop_limit: u32,
    vertex: Vertex,
) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)> {
    // create vec of out neighbors once and reuse it afterwards
    let out_neighbors = graph
        .out_graph()
        .edges(vertex)
        .map(|edge| edge.head)
        .collect_vec();

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
                    if let Some(current_edge_weight) = graph.get_weight(&edge.remove_weight()) {
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

pub fn edge_difference<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    new_and_updated_edges: &HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>,
    vertex: Vertex,
) -> i32 {
    new_and_updated_edges
        .values()
        .map(|(new_edges, _updated_edges)| new_edges.len() as i32)
        .sum::<i32>()
        - graph.in_graph().edges(vertex).len() as i32
        - graph.out_graph().edges(vertex).len() as i32
}
