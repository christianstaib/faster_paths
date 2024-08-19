use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::{Arc, Mutex},
};

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    graphs::{
        self, reversible_graph::ReversibleGraph, Distance, Edge, Graph, TaillessEdge, Vertex,
        WeightedEdge,
    },
    search::{
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

pub fn contraction_with_witness_search<G: Graph + Default>(
    mut graph: ReversibleGraph<G>,
) -> (Vec<Vertex>, HashMap<(Vertex, Vertex), Distance>) {
    println!("setting up the queue");
    let mut queue: BinaryHeap<Reverse<(i32, Vertex)>> = (0..graph.out_graph().number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let new_and_updated_edges = par_simulate_contraction_witness_search(&graph, vertex);
            let edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
            Reverse((edge_difference, vertex))
        })
        .collect();

    println!("contracting");
    let mut shortcuts = HashMap::new();
    for vertex in 0..graph.out_graph().number_of_vertices() {
        for edge in graph.out_graph().edges(vertex) {
            shortcuts.insert((edge.tail, edge.head), edge.weight);
        }
    }

    let mut level_to_vertex = Vec::new();

    let pb = ProgressBar::new(graph.out_graph().number_of_vertices() as u64);
    while let Some(Reverse((old_edge_difference, vertex))) = queue.pop() {
        let new_and_updated_edges = par_simulate_contraction_witness_search(&graph, vertex);
        let new_edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
        if new_edge_difference > old_edge_difference {
            queue.push(Reverse((new_edge_difference, vertex)));
            continue;
        }
        pb.inc(1);

        for (&tail, (new_edges, updated_edges)) in new_and_updated_edges.iter() {
            for edge in new_edges.iter().chain(updated_edges.iter()) {
                shortcuts.insert((tail, edge.head), edge.weight);
            }
        }

        level_to_vertex.push(vertex);
        graph.disconnect(vertex);
        graph.insert_and_update(&new_and_updated_edges);
    }
    pb.finish();

    (level_to_vertex, shortcuts)
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_witness_search<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
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
                        updated_edges.push(edge.remove_tail());
                    } else {
                        new_edges.push(edge.remove_tail());
                    }
                }
            });

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

pub fn simulate_contraction_distance_heuristic<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    distance_heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> (Vec<WeightedEdge>, Vec<WeightedEdge>) {
    let new_edges = Arc::new(Mutex::new(Vec::new()));
    let updated_edges = Arc::new(Mutex::new(Vec::new()));

    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .par_bridge()
        .for_each(|in_edge| {
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
