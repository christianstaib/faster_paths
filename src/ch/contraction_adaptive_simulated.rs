use std::time::Instant;

use ahash::HashSet;
use indicatif::{ParallelProgressIterator, ProgressBar};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use super::{
    contractor::{
        contraction_helper::ShortcutGeneratorWithWittnessSearch,
        serial_witness_search_contractor::SerialAdaptiveSimulatedContractor,
    },
    helpers::generate_directed_contracted_graph,
    priority_function::decode_function,
};
use crate::{
    ch::{
        ch_priority_element::ChPriorityElement, directed_contracted_graph::DirectedContractedGraph,
    },
    graphs::{
        edge::{Edge, WeightedEdge},
        graph_functions::{all_edges, neighbors},
        path::{PathFinding, ShortestPathRequest},
        reversible_vec_graph::ReversibleVecGraph,
        vec_graph::VecGraph,
        Graph, VertexId,
    },
    queue,
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

pub fn contract(graph: &mut ReversibleVecGraph, vertex: VertexId, pathfinder: &dyn PathFinding) {
    let in_edges = graph.in_edges[vertex as usize].clone();
    let out_edges = graph.out_edges[vertex as usize].clone();

    in_edges.iter().for_each(|in_edge| {
        let tail = in_edge.tail();
        out_edges.iter().for_each(|out_edge| {
            let head = out_edge.head();

            if tail != head {
                let request = ShortestPathRequest::new(tail, head).unwrap();
                let alternative_weight = pathfinder
                    .shortest_path_weight(&request)
                    .unwrap_or(u32::MAX);

                if (in_edge.weight() + out_edge.weight()) <= alternative_weight {
                    // edge allready exists
                    let edge = WeightedEdge::new(tail, head, in_edge.weight() + out_edge.weight())
                        .unwrap();
                    graph.set_edge(&edge);
                }
            }
        });
    });

    graph.remove_vertex(vertex);
}

pub fn edge_difference_all_in(
    graph: &ReversibleVecGraph,
    vertex: VertexId,
    pathfinder: &dyn PathFinding,
) -> i32 {
    let number_of_new_edges: usize = graph.in_edges[vertex as usize]
        .par_iter()
        .map(|in_edge| {
            let tail = in_edge.tail();
            graph.out_edges[vertex as usize]
                .iter()
                .filter(|&out_edge| {
                    let head = out_edge.head();

                    if tail != head {
                        let request = ShortestPathRequest::new(tail, head).unwrap();
                        let alternative_weight = pathfinder
                            .shortest_path_weight(&request)
                            .unwrap_or(u32::MAX);

                        if (in_edge.weight() + out_edge.weight()) <= alternative_weight {
                            // edge allready exists
                            return true;
                        }
                    }

                    false
                })
                .count()
        })
        .sum();

    number_of_new_edges as i32
        - graph.in_edges[vertex as usize].len() as i32
        - graph.out_edges[vertex as usize].len() as i32
}

pub fn contract_adaptive_simulated_all_in(
    graph: &dyn Graph,
    pathfinder: &dyn PathFinding,
) -> DirectedContractedGraph {
    let mut work_graph = ReversibleVecGraph::from_edges(&all_edges(graph));

    // shuffle vertices for smooth progress bar
    let mut vertices = (0..work_graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    println!("initalizing queue");
    let start = Instant::now();
    let mut queue: Vec<_> = vertices
        .par_iter()
        .progress()
        .map(|&vertex| {
            let priority = edge_difference_all_in(&work_graph, vertex, pathfinder);
            ChPriorityElement { vertex, priority }
        })
        .collect();
    println!("queue init took {:?}", start.elapsed());

    queue.par_sort_by_key(|elem| -elem.priority);

    let pb = ProgressBar::new(graph.number_of_vertices() as u64);
    while let Some(ChPriorityElement { vertex, .. }) = queue.pop() {
        pb.inc(1);
        let neighbors = neighbors(vertex, graph);

        contract(&mut work_graph, vertex, pathfinder);

        queue.par_iter_mut().for_each(|elem| {
            if neighbors.contains(&elem.vertex) {
                elem.priority = edge_difference_all_in(&work_graph, vertex, pathfinder);
            }
        });

        queue.par_sort_by_key(|elem| -elem.priority);
    }

    todo!()
}
