use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Vertex, WeightedEdge,
    },
    search::{
        ch::{
            contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
            contraction::contraction_with_witness_search,
        },
        dijkstra::{create_test_cases, dijkstra_one_to_one_wrapped},
        path::{ShortestPathRequest, ShortestPathTestCase},
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
}

fn large_test_graph() -> (ReversibleGraph<VecVecGraph>, Vec<ShortestPathTestCase>) {
    let edges = read_edges_from_fmi_file(Path::new("tests/data/stgtregbz_cutout.fmi"));
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let reader = BufReader::new(File::open("test_cases.json").unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    (graph, test_cases)
}

fn main() {
    let (graph, test_cases) = large_test_graph();

    let out_graph = graph.out_graph().clone();

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_witness_search(graph);
    let contracted_graph = create_contracted_graph(edges, &level_to_vertex);

    let speedup = test_cases
        .iter()
        .progress()
        .map(
            |ShortestPathTestCase {
                 request: ShortestPathRequest { source, target },
                 distance,
             }| {
                let start = Instant::now();
                let ch_distance = ch_one_to_one_wrapped(&contracted_graph, *source, *target);
                let ch_time = start.elapsed().as_secs_f64();

                let start = Instant::now();
                let dijkstra_distance = dijkstra_one_to_one_wrapped(&out_graph, *source, *target);
                let dijkstra_time = start.elapsed().as_secs_f64();

                assert_eq!(distance, &dijkstra_distance);
                assert_eq!(distance, &ch_distance);

                dijkstra_time / ch_time
            },
        )
        .collect_vec();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}

fn create_contracted_graph(
    edges: HashMap<(Vertex, Vertex), Distance>,
    level_to_vertex: &Vec<u32>,
) -> ContractedGraph {
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let mut upward_edges = Vec::new();
    let mut downward_edges = Vec::new();
    for (&(tail, head), &weight) in edges.iter() {
        if vertex_to_level[tail as usize] < vertex_to_level[head as usize] {
            upward_edges.push(WeightedEdge::new(tail, head, weight));
        } else if vertex_to_level[tail as usize] > vertex_to_level[head as usize] {
            downward_edges.push(WeightedEdge::new(head, tail, weight));
        } else {
            panic!("tail and head have same level");
        }
    }

    ContractedGraph {
        upward_graph: VecVecGraph::from_edges(&upward_edges),
        downward_graph: VecVecGraph::from_edges(&downward_edges),
        level_to_vertex: level_to_vertex.clone(),
        vertex_to_level,
    }
}

pub fn vertex_to_level(level_to_vertex: &Vec<Vertex>) -> Vec<u32> {
    let mut vertex_to_level = vec![0; level_to_vertex.len()];

    for (level, &vertex) in level_to_vertex.iter().enumerate() {
        vertex_to_level[vertex as usize] = level as u32;
    }

    vertex_to_level
}
