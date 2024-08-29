use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use super::contraction::edge_difference;
use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, Distance, Graph, TaillessEdge, Vertex, WeightedEdge,
    },
    search::{
        ch::contraction::{new_edge_map, update_edge_map},
        DistanceHeuristic,
    },
    utility::get_progressbar_long_jobs,
};

pub fn contraction_with_heuristic<G: Graph + Default>(
    mut graph: ReversibleGraph<G>,
    heuristic: &dyn DistanceHeuristic,
) -> (
    Vec<Vertex>,
    HashMap<(Vertex, Vertex), Distance>,
    HashMap<(Vertex, Vertex), Vertex>,
) {
    println!("setting up the queue");
    let mut queue = new_queue(&graph, heuristic);

    println!("contracting");
    let mut edges = new_edge_map(&graph);
    let mut shortcuts = HashMap::new();

    let mut level_to_vertex = Vec::new();

    let pb = get_progressbar_long_jobs("Contration", graph.out_graph().number_of_vertices() as u64);
    while let Some(Reverse((old_edge_difference, vertex))) = queue.pop() {
        let new_and_updated_edges = par_simulate_contraction_heuristic(&graph, heuristic, vertex);
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

fn new_queue<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    heuristic: &dyn DistanceHeuristic,
) -> BinaryHeap<Reverse<(i32, u32)>> {
    let mut vertices = (0..graph.out_graph().number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    vertices
        .into_par_iter()
        .progress_with(get_progressbar_long_jobs(
            "Initalizing queue",
            graph.out_graph().number_of_vertices() as u64,
        ))
        .map(|vertex| {
            let new_and_updated_edges =
                par_simulate_contraction_heuristic(&graph, heuristic, vertex);
            let edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
            Reverse((edge_difference, vertex))
        })
        .collect()
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_heuristic<G: Graph + Default>(
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

                if heuristic.is_less_or_equal_upper_bound(tail, head, shortcut_distance) {
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

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_heuristic_new_edges<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> u32 {
    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .par_bridge()
        .map(|in_edge| {
            let tail = in_edge.head;

            let mut new_edges = 0;

            for out_edge in graph.out_graph().edges(vertex) {
                let head = out_edge.head;

                if tail == head {
                    continue;
                }

                let shortcut_distance = in_edge.weight + out_edge.weight;

                if heuristic.is_less_or_equal_upper_bound(tail, head, shortcut_distance) {
                    let edge = WeightedEdge {
                        tail,
                        head,
                        weight: shortcut_distance,
                    };
                    if let Some(_current_edge_weight) = graph.get_weight(&edge.remove_weight()) {
                    } else {
                        new_edges += 1;
                    }
                }
            }

            new_edges
        })
        .sum()
}
