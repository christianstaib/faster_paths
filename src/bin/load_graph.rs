use std::{cmp::Reverse, collections::BinaryHeap, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Graph, Vertex,
    },
    search::ch::contraction::{edge_difference, simulate_contraction},
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

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

    let mut graph = ReversibleGraph::<VecVecGraph>::new();

    println!("Building graph");
    edges.into_iter().progress().for_each(|edge| {
        if edge.weight
            < graph
                .get_weight(&edge.remove_weight())
                .unwrap_or(Distance::MAX)
        {
            graph.set_weight(&edge.remove_weight(), Some(edge.weight));
        }
    });

    let mut vertices = (0..graph.out_graph().number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    println!("set up ch queue");
    let mut queue: BinaryHeap<Reverse<(i32, Vertex)>> = vertices
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (new_edges, _updated_edges) = simulate_contraction(&graph, vertex);
            let edge_difference = edge_difference(&graph, &new_edges, vertex);
            Reverse((edge_difference, vertex))
        })
        .collect();

    println!("min edge difference is {:?}", queue.pop().unwrap());

    // println!("Reading test cases");
    // let mut reader = BufReader::new(File::open(&args.tests).unwrap());
    // let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(&mut
    // reader).unwrap();
    // let graph = graph.out_graph();
    // let graph = &graph;
    // for _ in (0..200).progress().progress() {
    //     let source = thread_rng().gen_range(0..graph.number_of_vertices());
    //     let target = thread_rng().gen_range(0..graph.number_of_vertices());

    //     let mut data = DijkstraDataVec::new(graph);
    //     let mut expanded = VertexExpandedDataBitSet::new(graph);
    //     let mut queue = VertexDistanceQueueDaryHeap::<3>::new();

    //     let start = Instant::now();
    //     dijktra_one_to_one(graph, &mut data, &mut expanded, &mut queue,
    // source, target);     duration += start.elapsed();
    // }
    // println!(
    //     "average duration was {:?}",
    //     duration / (test_cases.len() as u32)
    // );
}