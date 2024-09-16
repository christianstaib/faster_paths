use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_hash_graph::VecHashGraph,
        vec_vec_graph::VecVecGraph, Distance, Graph, Vertex, WeightedEdge,
    },
    search::{
        ch::contracted_graph::ContractedGraph, hl::hub_graph::HubGraph, DistanceHeuristic,
        PathFinding,
    },
    utility::{
        benchmark_and_test_distance, generate_test_cases, get_paths, get_progressbar, hitting_set,
        level_to_vertex, read_bincode_with_spinnner, write_json_with_spinnner,
    },
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph_hl: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    ch: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph_org: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());
    let mut graph: ReversibleGraph<VecHashGraph> =
        ReversibleGraph::from_edges(&graph_org.out_graph().all_edges());

    let simple_graph_hl: HubGraph =
        read_bincode_with_spinnner("simple hub graph", &args.simple_graph_hl);

    println!(
        "{} comparisions needed",
        graph
            .out_graph()
            .vertices()
            .map(|vertex| graph.out_graph().edges(vertex).len()
                * graph.in_graph().edges(vertex).len())
            .sum::<usize>()
    );
    let mut edges = Vec::new();

    let upper_lim_degree = 2_000;

    let mut vertices = graph
        .out_graph()
        .vertices()
        .progress()
        .par_bridge()
        .filter(|&vertex| graph_org.out_graph().edges(vertex).len() < upper_lim_degree)
        .map(|vertex| {
            let potentially_new_edges = potentially_new_edges(&graph, &simple_graph_hl, vertex);
            (
                vertex,
                edge_difference(&graph, vertex, &potentially_new_edges),
            )
        })
        .collect::<HashMap<Vertex, i32>>();

    let mut level_to_vertex = Vec::new();
    let pb = get_progressbar("Contracting", graph.number_of_vertices() as u64);
    while !vertices.is_empty() {
        // let len_vertices = vertices.len() as u64;
        // vertices = vertices
        //     .into_par_iter()
        //     .progress_count(len_vertices)
        //     .filter(|&vertex| graph.out_graph().edges(vertex).len() < 2000)
        //     .collect::<HashSet<_>>();

        let vertex = *vertices
            .par_iter()
            .min_by_key(|&(_vertex, diff)| diff)
            .unwrap()
            .0;

        vertices.remove(&vertex);
        level_to_vertex.push(vertex);

        let neighbors = graph
            .out_graph()
            .edges(vertex)
            .map(|edge| edge.head)
            .collect::<HashSet<_>>();

        let len_vertices = vertices.len() as u64;
        vertices = vertices
            .into_par_iter()
            .filter(|&(vertex, _old_diff)| graph.out_graph().edges(vertex).len() < upper_lim_degree)
            .map(|(vertex, old_diff)| {
                if neighbors.contains(&vertex) {
                    let potentially_new_edges =
                        potentially_new_edges(&graph, &simple_graph_hl, vertex);
                    return (
                        vertex,
                        edge_difference(&graph, vertex, &potentially_new_edges),
                    );
                }

                (vertex, old_diff)
            })
            .collect::<HashMap<_, _>>();

        let potentially_new_edges = potentially_new_edges(&mut graph, &simple_graph_hl, vertex);
        edges.extend(contract(&mut graph, vertex, &potentially_new_edges));

        pb.inc(1);
    }
    pb.finish_and_clear();

    assert!(graph.out_graph().all_edges().is_empty());

    let ch = ContractedGraph::new(level_to_vertex, edges, HashMap::new());

    // Benchmark and test correctness
    let tests = generate_test_cases(graph_org.out_graph(), 1_000);
    let average_duration = benchmark_and_test_distance(&tests, &ch).unwrap();
    println!("Average duration was {:?}", average_duration);
}

pub fn potentially_new_edges<T: Graph>(
    graph: &ReversibleGraph<T>,
    heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> Vec<WeightedEdge> {
    let in_edges = graph.in_graph().edges(vertex).collect_vec();
    let out_edges = graph.out_graph().edges(vertex).collect_vec();

    in_edges
        .par_iter()
        .map(|in_edge| {
            out_edges
                .iter()
                .flat_map(|out_edge| {
                    let alt_weight = in_edge.weight + out_edge.weight;
                    if alt_weight <= heuristic.upper_bound(in_edge.head, out_edge.head) {
                        return Some(WeightedEdge::new(in_edge.head, out_edge.head, alt_weight));
                    }
                    None
                })
                .collect_vec()
        })
        .flatten()
        .collect()
}

pub fn edge_difference<T: Graph>(
    graph: &ReversibleGraph<T>,
    vertex: Vertex,
    potentially_new_edges: &Vec<WeightedEdge>,
) -> i32 {
    let mut new_edges = 0;

    for edge in potentially_new_edges {
        if edge.weight
            < graph
                .get_weight(&edge.remove_weight())
                .unwrap_or(Distance::MAX)
        {
            new_edges += 1;
        }
    }

    new_edges
        - graph.in_graph().edges(vertex).len() as i32
        - graph.out_graph().edges(vertex).len() as i32
}

pub fn contract<T: Graph>(
    graph: &mut ReversibleGraph<T>,
    vertex: Vertex,
    potentially_new_edges: &Vec<WeightedEdge>,
) -> Vec<WeightedEdge> {
    let mut edges = graph.out_graph().edges(vertex).collect_vec();
    edges.extend(graph.in_graph().edges(vertex).map(|edge| edge.reversed()));

    graph.disconnect(vertex);

    for edge in potentially_new_edges {
        if edge.weight
            < graph
                .get_weight(&edge.remove_weight())
                .unwrap_or(Distance::MAX)
        {
            graph.set_weight(&edge.remove_weight(), Some(edge.weight));
        }
    }

    edges
}
