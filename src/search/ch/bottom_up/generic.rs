use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use log::info;
use rand::{prelude::*, thread_rng};
use rayon::prelude::*;

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Graph, Level, TaillessEdge, Vertex, WeightedEdge},
    utility::get_progressbar,
};

pub fn contraction_top_down<G, F>(
    mut graph: ReversibleGraph<G>,
    level_to_vertex: &Vec<Level>,
    shortcut_generation: F,
) -> (
    Vec<Vertex>,
    Vec<WeightedEdge>,
    HashMap<(Vertex, Vertex), Vertex>,
)
where
    G: Graph,
    F: Fn(&ReversibleGraph<G>, Vertex) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>
        + Send
        + Sync,
{
    let mut edges = graph
        .out_graph()
        .vertices()
        .flat_map(|vertex| graph.out_graph().edges(vertex))
        .collect();

    let mut shortcuts = HashMap::new();

    let number_of_vertices = graph.out_graph().number_of_vertices() as u64;
    let pb = get_progressbar("Contracting", number_of_vertices);

    for &vertex in level_to_vertex.iter().progress_with(pb) {
        let new_and_updated_edges = shortcut_generation(&graph, vertex);
        update_edge_map(&mut edges, &mut shortcuts, vertex, &new_and_updated_edges);
        graph.disconnect(vertex);
        graph.insert_and_update(&new_and_updated_edges);
    }

    (level_to_vertex.clone(), edges, shortcuts)
}

pub fn contraction_bottom_up<G, F>(
    mut graph: ReversibleGraph<G>,
    shortcut_generation: F,
) -> (
    Vec<Vertex>,
    Vec<WeightedEdge>,
    HashMap<(Vertex, Vertex), Vertex>,
)
where
    G: Graph,
    F: Fn(&ReversibleGraph<G>, Vertex) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>
        + Send
        + Sync,
{
    info!("Setting up queue");
    let mut queue = new_queue_generic(&graph, &shortcut_generation);

    let mut edges = new_edge_map(&graph);
    let mut shortcuts = HashMap::new();

    let mut level_to_vertex = Vec::new();

    let number_of_vertices = graph.out_graph().number_of_vertices() as u64;
    let pb = get_progressbar("Contracting", number_of_vertices);

    info!("Start contracting");
    while let Some(Reverse((old_edge_difference, vertex))) = queue.pop() {
        info!(
            "Contracting {}. {:>2.2}% remaining",
            vertex,
            queue.len() as f32 / number_of_vertices as f32 * 100.0
        );
        let new_and_updated_edges = shortcut_generation(&graph, vertex);
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
    info!("Finished contracting");

    (level_to_vertex, edges, shortcuts)
}

pub fn new_queue_generic<G, F>(
    graph: &ReversibleGraph<G>,
    shortcut_generation: F,
) -> BinaryHeap<Reverse<(i32, u32)>>
where
    G: Graph,
    F: Fn(&ReversibleGraph<G>, Vertex) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>
        + Send
        + Sync,
{
    let pb = get_progressbar(
        "Initializing queue",
        graph.out_graph().number_of_vertices() as u64,
    );

    let mut vertices = graph.out_graph().vertices().collect_vec();
    vertices.shuffle(&mut thread_rng());

    vertices
        .into_par_iter()
        .progress_with(pb)
        .map(|vertex| {
            let new_and_updated_edges = shortcut_generation(&graph, vertex);
            let edge_difference = edge_difference(graph, &new_and_updated_edges, vertex);
            Reverse((edge_difference, vertex))
        })
        .collect()
}

pub fn update_edge_map(
    edge_map: &mut Vec<WeightedEdge>,
    shortcuts: &mut HashMap<(Vertex, Vertex), Vertex>,
    vertex: Vertex,
    new_and_updated_edges: &HashMap<u32, (Vec<TaillessEdge>, Vec<TaillessEdge>)>,
) {
    for (&tail, (new_edges, updated_edges)) in new_and_updated_edges.iter() {
        for edge in new_edges.iter().chain(updated_edges.iter()) {
            assert_ne!(tail, edge.head);
            assert_ne!(edge.head, vertex);
            assert_ne!(tail, vertex);

            edge_map.push(edge.set_tail(tail));
            shortcuts.insert((tail, edge.head), vertex);
        }
    }
}

pub fn edge_difference<G: Graph>(
    graph: &ReversibleGraph<G>,
    new_and_updated_edges: &HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>,
    vertex: Vertex,
) -> i32 {
    let mut neighbors_newedges_map = graph
        .out_graph()
        .edges(vertex)
        .map(|x| (x.head, -1))
        .collect::<HashMap<_, _>>();

    for (x, (new, _)) in new_and_updated_edges.iter() {
        *neighbors_newedges_map.get_mut(x).unwrap() += new.len() as i64;
    }

    let p = 2;

    neighbors_newedges_map
        .iter()
        .map(|(n, diff)| {
            let old = graph.out_graph().edges(*n).len() as i64;

            (old + diff).pow(p) - old.pow(p)
        })
        .sum::<i64>() as i32
}

pub fn new_edge_map<G: Graph>(graph: &ReversibleGraph<G>) -> Vec<WeightedEdge> {
    graph
        .out_graph()
        .vertices()
        .flat_map(|vertex| graph.out_graph().edges(vertex))
        .collect()
}
