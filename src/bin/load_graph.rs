use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Graph, Vertex,
    },
    search::{
        alt::landmark::Landmarks,
        ch::contraction::{
            probabilistic_edge_difference_distance_neuristic,
            simulate_contraction_distance_heuristic,
        },
    },
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

    println!("Generating Landmarks");
    let landmarks = Landmarks::new(
        &graph,
        &vertices
            .choose_multiple(&mut thread_rng(), 23)
            .cloned()
            .collect_vec(),
    );

    println!("set up ch queue");
    let mut queue: BinaryHeap<Reverse<(i32, Vertex)>> = vertices
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let edge_difference = probabilistic_edge_difference_distance_neuristic(
                &graph, &landmarks, vertex, 50, 5000, 0.1,
            );
            Reverse((edge_difference, vertex))
        })
        .collect();
    println!("min edge difference is {:?}", queue.pop().unwrap());

    let mut neighbors_hits = (0..graph.out_graph().number_of_vertices())
        .map(|vertex| 0)
        .collect_vec();
    while let Some(Reverse((_priority, vertex))) = queue.pop() {
        println!("queue len is {}", queue.len());

        println!("getting neighbors");
        let mut to_update = HashSet::new();
        graph
            .out_graph()
            .edges(vertex)
            .chain(graph.in_graph().edges(vertex))
            .for_each(|edge| {
                neighbors_hits[edge.head as usize] += 1;
                if neighbors_hits[edge.head as usize] >= 100 {
                    to_update.insert(edge.head);
                    neighbors_hits[edge.head as usize] = 0;
                }
            });

        println!("contracting");
        let (new_edges, updated_edges) =
            simulate_contraction_distance_heuristic(&graph, &landmarks, vertex);

        println!("Removing vertex");
        let mut to_remove = graph.out_graph().edges(vertex).collect_vec();
        to_remove.extend(graph.in_graph().edges(vertex).map(|edge| edge.reversed()));

        for edge in to_remove.into_iter().progress() {
            graph.set_weight(&edge.remove_weight(), None);
        }

        println!("adding vertices");
        let x = new_edges.len() as u64 + updated_edges.len() as u64;
        for edge in new_edges
            .into_iter()
            .chain(updated_edges.into_iter())
            .progress_count(x)
        {
            if edge.weight < graph.get_weight(&edge.remove_weight()).unwrap_or(u32::MAX) {
                graph.set_weight(&edge.remove_weight(), Some(edge.weight));
            }
        }

        println!("updating queue");
        queue = queue
            .into_par_iter()
            .progress()
            .map(|Reverse((mut priority, vertex))| {
                if to_update.contains(&vertex) {
                    priority = probabilistic_edge_difference_distance_neuristic(
                        &graph, &landmarks, vertex, 50, 5000, 0.1,
                    );
                }

                Reverse((priority, vertex))
            })
            .collect();
    }

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
