use std::{collections::HashMap, path::PathBuf, time::Instant};

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
        path::ShortestPathTestCase,
    },
};
use indicatif::ProgressIterator;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Reading edges");
    let edges = read_edges_from_fmi_file(&args.graph);
    let out_graph = VecVecGraph::from_edges(&edges);

    println!("Building graph");
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!("Creating test cases");
    let test_cases = create_test_cases(graph.out_graph(), 100_000);

    println!("Create contracted graph");
    let (level_to_vertex, shortcuts) = contraction_with_witness_search(graph);
    let contracted_graph = create_contracted_graph(shortcuts, &edges, &level_to_vertex);

    let mut speedup = Vec::new();
    for ShortestPathTestCase { request, distance } in test_cases.iter().progress() {
        let source = request.source;
        let target = request.target;

        let start = Instant::now();
        let ch_distance = ch_one_to_one_wrapped(&contracted_graph, source, target);
        let ch_time = start.elapsed().as_secs_f64();

        let start = Instant::now();
        let dijkstra_distance = dijkstra_one_to_one_wrapped(&out_graph, source, target);
        let dijkstra_time = start.elapsed().as_secs_f64();

        assert_eq!(distance, &dijkstra_distance);
        assert_eq!(distance, &ch_distance);

        speedup.push(dijkstra_time / ch_time);
    }

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}

fn create_contracted_graph(
    shortcuts: HashMap<(Vertex, Vertex), Distance>,
    edges: &Vec<WeightedEdge>,
    level_to_vertex: &Vec<u32>,
) -> ContractedGraph {
    let mut edges = edges.clone();

    shortcuts.iter().for_each(|(&(tail, head), &weight)| {
        let edge = WeightedEdge { tail, head, weight };
        edges.push(edge);
    });

    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let mut upward_edges = Vec::new();
    let mut downward_edges = Vec::new();
    for edge in edges.iter() {
        if vertex_to_level[edge.tail as usize] < vertex_to_level[edge.head as usize] {
            upward_edges.push(edge.clone())
        } else {
            downward_edges.push(edge.reversed())
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
