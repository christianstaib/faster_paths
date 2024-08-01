use std::{
    collections::BinaryHeap,
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use super::{
    contractor::{
        contraction_helper::{
            ShortcutGenerator, ShortcutGeneratorWithHeuristic, ShortcutGeneratorWithWittnessSearch,
        },
        serial_witness_search_contractor::SerialAdaptiveSimulatedContractor,
    },
    helpers::generate_directed_contracted_graph,
    priority_function::decode_function,
};
use crate::{
    ch::{
        ch_priority_element::ChPriorityElement, directed_contracted_graph::DirectedContractedGraph,
        Shortcut,
    },
    graphs::{
        self,
        edge::{Edge, WeightedEdge},
        graph_functions::all_edges,
        reversible_graph::ReversibleGraph,
        reversible_hash_graph::ReversibleHashGraph,
        reversible_vec_graph::ReversibleVecGraph,
        vec_graph::VecGraph,
        Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub fn contract_adaptive_simulated_with_witness(graph: &dyn Graph) -> DirectedContractedGraph {
    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    let priority_terms = decode_function("E:1_D:1_C:1");

    let shortcut_generator = ShortcutGeneratorWithWittnessSearch { max_hops: 16 };
    let mut contractor =
        SerialAdaptiveSimulatedContractor::new(priority_terms, &shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    generate_directed_contracted_graph(vec_graph, &shortcuts, &levels)
}

pub fn edge_difference_all_in(
    graph: &ReversibleVecGraph,
    vertex: VertexId,
    edges: &Vec<HashSet<VertexId>>,
) -> i32 {
    let number_of_new_edges: usize = graph.in_edges[vertex as usize]
        .par_iter()
        .map(|in_edge| {
            let tail = in_edge.tail();
            graph.out_edges[vertex as usize]
                .iter()
                .filter(|&out_edge| {
                    let head = out_edge.head();

                    // if graph.out_edges[tail as usize]
                    //     .binary_search_by_key(&head, |edge| edge.head())
                    //     .is_ok()
                    // {
                    if edges[tail as usize].contains(&head) {
                        // edge allready exists
                        return false;
                    }

                    true
                })
                .count()
        })
        .sum();

    number_of_new_edges as i32
        - graph.in_edges[vertex as usize].len() as i32
        - graph.out_edges[vertex as usize].len() as i32
}

pub fn contract_adaptive_simulated_all_in(graph: &dyn Graph) -> DirectedContractedGraph {
    let mut work_graph = ReversibleVecGraph::from_edges(&all_edges(graph));

    // shuffle vertices for smooth progress bar
    let mut vertices = (0..work_graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    let hash_graph: Vec<HashSet<VertexId>> = (0..graph.number_of_vertices())
        .map(|vertex| graph.out_edges(vertex).map(|edge| edge.head()).collect())
        .collect();
    println!("initalizing queue");
    let start = Instant::now();
    let mut queue: Vec<_> = vertices
        .par_iter()
        .progress()
        .map(|&vertex| {
            let priority = edge_difference_all_in(&work_graph, vertex, &hash_graph);
            ChPriorityElement { vertex, priority }
        })
        .collect();
    println!("queue init took {:?}", start.elapsed());

    let minmax = queue
        .iter()
        .minmax_by_key(|entry| entry.priority)
        .into_option()
        .unwrap();

    println!(
        "min max edge difference is {} {}",
        minmax.0.priority, minmax.1.priority,
    );

    todo!()
}
