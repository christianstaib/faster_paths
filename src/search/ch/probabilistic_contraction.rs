use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use indicatif::{ParallelProgressIterator, ProgressBar};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, Distance, Edge, Graph, TaillessEdge, Vertex,
        WeightedEdge,
    },
    search::DistanceHeuristic,
};

fn scale(base_value: u32, factor: f32, min: u32, max: u32) -> u32 {
    let scaled_value = (base_value as f32 * factor).round() as u32;
    scaled_value.clamp(min, max)
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

pub fn contraction_with_distance_heuristic<G: Graph + Default>(
    mut graph: ReversibleGraph<G>,
    distance_heuristic: &dyn DistanceHeuristic,
) -> (Vec<Vertex>, HashMap<(Vertex, Vertex), Distance>) {
    println!("setting up the queue");
    let mut queue: BinaryHeap<Reverse<(i32, Vertex)>> = (0..graph.out_graph().number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let new_and_updated_edges =
                par_simulate_contraction_distance_heuristic(&graph, distance_heuristic, vertex);
            let edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
            Reverse((edge_difference, vertex))
        })
        .collect();

    println!("contracting");
    let mut edges = HashMap::new();
    for vertex in 0..graph.out_graph().number_of_vertices() {
        for edge in graph.out_graph().edges(vertex) {
            edges.insert((edge.tail, edge.head), edge.weight);
        }
    }

    let mut level_to_vertex = Vec::new();

    let mut neighbors = (0..graph.out_graph().number_of_vertices())
        .map(|_| 0)
        .collect_vec();

    let pb = ProgressBar::new(graph.out_graph().number_of_vertices() as u64);

    let mut writer = BufWriter::new(File::create("time.csv").unwrap());
    writeln!(
        writer,
        "create edges,to update,update edge map,disconnect,insert and update,update queue"
    )
    .unwrap();
    while let Some(Reverse((_edge_difference, vertex))) = queue.pop() {
        // pb.inc(1);

        let start = Instant::now();
        let new_and_updated_edges =
            par_simulate_contraction_distance_heuristic(&graph, distance_heuristic, vertex);
        let p0 = start.elapsed().as_secs_f64();

        let start = Instant::now();
        let to_update = get_to_update(&graph, vertex, &mut neighbors);
        let p1 = start.elapsed().as_secs_f64();

        let start = Instant::now();
        update_edges_map(&new_and_updated_edges, &mut edges);
        let p2 = start.elapsed().as_secs_f64();

        level_to_vertex.push(vertex);
        let start = Instant::now();
        graph.disconnect(vertex);
        let p3 = start.elapsed().as_secs_f64();
        let start = Instant::now();
        graph.insert_and_update(&new_and_updated_edges);
        let p4 = start.elapsed().as_secs_f64();

        let start = Instant::now();
        queue = update_queue(queue, to_update, &graph, distance_heuristic);
        let p5 = start.elapsed().as_secs_f64();

        writeln!(writer, "{},{},{},{},{},{}", p0, p1, p2, p3, p4, p5).unwrap();
        writer.flush().unwrap();
    }
    pb.finish();

    (level_to_vertex, edges)
}

fn get_to_update<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    vertex: u32,
    neighbors: &mut Vec<i32>,
) -> Vec<u32> {
    let mut to_update = Vec::new();
    for edge in graph.out_graph().edges(vertex) {
        neighbors[edge.head as usize] += 1;

        if neighbors[edge.head as usize] >= 10 {
            neighbors[edge.head as usize] = 0;
            to_update.push(edge.head);
        }
    }
    to_update
}

fn update_edges_map(
    new_and_updated_edges: &HashMap<u32, (Vec<TaillessEdge>, Vec<TaillessEdge>)>,
    edges: &mut HashMap<(Vertex, Vertex), Distance>,
) {
    for (&tail, (new_edges, updated_edges)) in new_and_updated_edges.iter() {
        for edge in new_edges.iter().chain(updated_edges.iter()) {
            edges.insert((tail, edge.head), edge.weight);
        }
    }
}

fn update_queue<G: Graph + Default>(
    queue: BinaryHeap<Reverse<(i32, Vertex)>>,
    to_update: Vec<u32>,
    graph: &ReversibleGraph<G>,
    distance_heuristic: &dyn DistanceHeuristic,
) -> BinaryHeap<Reverse<(i32, u32)>> {
    queue
        .into_par_iter()
        .map(|Reverse((old_edge_difference, vertex))| {
            if to_update.contains(&vertex) {
                let new_and_updated_edges =
                    par_simulate_contraction_distance_heuristic(graph, distance_heuristic, vertex);
                let edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
                return Reverse((edge_difference, vertex));
            }
            Reverse((old_edge_difference, vertex))
        })
        .collect()
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_distance_heuristic<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    distance_heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)> {
    // tail -> vertex -> head
    graph
        .in_graph()
        .edges(vertex)
        .par_bridge()
        .map(|in_edge| {
            // an edge head in the in_graph is a tail in the out_graph
            let tail = in_edge.head;

            let mut new_edges = Vec::new();
            let mut updated_edges = Vec::new();

            for out_edge in graph.out_graph().edges(vertex) {
                let head = out_edge.head;

                // (tail -> vertex -> head) cant be a shortest path if (tail == head)
                if tail == head {
                    continue;
                }

                let shortcut_distance = in_edge.weight + out_edge.weight;
                let edge = WeightedEdge::new(tail, head, shortcut_distance);

                // if there is already an edge (tail -> head) but shortcut_distance is less than
                // the current distance, update edge weight
                if let Some(current_distance) = graph.get_weight(&edge.remove_weight()) {
                    if shortcut_distance < current_distance {
                        updated_edges.push(edge.remove_tail());
                    }
                    continue;
                }

                // if shortcut_distance <= upper_bound_distance {
                if distance_heuristic.is_less_or_equal_upper_bound(tail, head, shortcut_distance) {
                    new_edges.push(edge.remove_tail());
                }
            }

            (tail, (new_edges, updated_edges))
        })
        .collect()
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
        return 0 - (in_edges.len() as i32) - (out_edges.len() as i32);
    }

    let searches = scale(
        number_of_edge_pairs,
        search_factor,
        std::cmp::min(number_of_edge_pairs as u32, min_searches),
        std::cmp::min(number_of_edge_pairs as u32, max_searches),
    );

    let searches_factor = searches as f32 / number_of_edge_pairs as f32;

    let mut rng = thread_rng();
    let mut new_edges_len = 0;
    for _ in 0..searches {
        let in_edge = in_edges.choose(&mut rng).unwrap();
        let out_edge = in_edges.choose(&mut rng).unwrap();

        let shortcut_distance = in_edge.weight + out_edge.weight;
        let lower_bound = distance_heuristic.upper_bound(in_edge.head, out_edge.head);

        if in_edge.head != out_edge.head && shortcut_distance <= lower_bound {
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
