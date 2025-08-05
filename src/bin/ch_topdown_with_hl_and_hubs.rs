use std::{
    path::PathBuf,
    sync::{atomic::AtomicI32, Arc, Mutex},
};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_hash_graph::VecHashGraph,
        vec_vec_graph::VecVecGraph, Distance, Graph, Vertex, WeightedEdge,
    },
    search::{
        alt::landmark::Landmarks,
        ch::{
            bottom_up::{generic::edge_difference, heuristic::par_simulate_contraction_heuristic},
            contracted_graph::ContractedGraph,
        },
        hl::hub_graph::{self, HubGraph},
        DistanceHeuristic, TrivialHeuristic,
    },
    utility::{
        benchmark_and_test_path, generate_test_cases, read_bincode_with_spinnner,
        read_json_with_spinnner, write_bincode_with_spinnner,
    },
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    level_to_vertex: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();
    env_logger::init();

    // Build graph
    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());
    let graph: ReversibleGraph<VecHashGraph> =
        ReversibleGraph::from_edges(&graph.out_graph().all_edges());

    let hub_graph: HubGraph = read_bincode_with_spinnner("hub graph", &args.hub_graph.as_path());

    let level_to_vertex: Vec<Vertex> =
        read_json_with_spinnner("level to vertex", &args.level_to_vertex.as_path());

    let landmarks = Landmarks::new(
        &graph,
        &level_to_vertex.iter().rev().take(80).cloned().collect_vec(),
    );

    // Create contracted_graph
    let combinded_heuristic = CombindedHeuristic {
        hub_graph,
        landmarks,
    };

    let min_edge_diff = AtomicI32::new(i32::MAX / 2);
    let mut diffs = graph
        .out_graph()
        .vertices()
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let alt_edge_diff = par_simulate_contraction_heuristic_break(
                &graph,
                &combinded_heuristic,
                vertex,
                min_edge_diff.load(std::sync::atomic::Ordering::Relaxed),
            );

            min_edge_diff.fetch_min(alt_edge_diff, std::sync::atomic::Ordering::Relaxed);

            alt_edge_diff
        })
        .collect::<Vec<_>>();

    diffs.sort_unstable();
    println!("{:?}", diffs.into_iter().take(25).collect_vec());

    let contracted_graph = ContractedGraph::by_contraction_top_down_with_heuristic(
        &graph,
        &level_to_vertex,
        &combinded_heuristic,
    );

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration =
        benchmark_and_test_path(graph.out_graph(), &tests, &contracted_graph).unwrap();
    println!("Average duration was {:?}", average_duration);

    // Write contracted_graph to file
    write_bincode_with_spinnner(
        "contracted_graph",
        &args.contracted_graph.as_path(),
        &contracted_graph,
    );
}

struct CombindedHeuristic {
    pub hub_graph: HubGraph,
    pub landmarks: Landmarks,
}

impl DistanceHeuristic for CombindedHeuristic {
    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool {
        self.hub_graph
            .is_less_or_equal_upper_bound(source, target, distance)
            && self
                .landmarks
                .is_less_or_equal_upper_bound(source, target, distance)
    }
}

/// Simulates a contraction. Returns vertex -> (new_edges, updated_edges)
pub fn par_simulate_contraction_heuristic_break<G: Graph>(
    graph: &ReversibleGraph<G>,
    heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
    min_edge_diff: i32,
) -> i32 {
    let num_remove_edges =
        graph.in_graph().edges(vertex).len() as i32 + graph.out_graph().edges(vertex).len() as i32;

    let max_allow_new_edges = min_edge_diff + num_remove_edges - 1;

    if max_allow_new_edges < 0 {
        return (i32::MAX) / 2;
    }

    let mut new_edges = Vec::new();
    // tail -> vertex -> head
    for in_edge in graph.in_graph().edges(vertex) {
        let tail = in_edge.head;

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

                // Checking current edge weight is propabally cheaper than heuristic so check
                // first
                if let Some(_current_edge_weight) =
                    graph.out_graph().get_weight(&edge.remove_weight())
                {
                    // if shortcut_distance < current_edge_weight {
                    //     updated_edges.push(edge.remove_tail());
                    // }
                    continue;
                }

                new_edges.push(edge.remove_tail());
                if new_edges.len() as i32 >= max_allow_new_edges {
                    return (i32::MAX) / 2;
                }
            }
        }
    }

    return new_edges.len() as i32 - num_remove_edges;
}
