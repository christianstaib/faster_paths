use std::{collections::HashSet, ops::Sub, path::PathBuf, process::exit, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, WeightedEdge,
    },
    search::{
        ch::{
            brute_force::{self, brute_force_contracted_graph, get_ch_edges},
            contracted_graph::{ch_one_to_one_wrapped, vertex_to_level, ContractedGraph},
            contraction::contraction_with_witness_search,
        },
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
        },
        dijkstra::{dijkstra_one_to_all_wraped, dijkstra_one_to_one, dijkstra_one_to_one_wrapped},
    },
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

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

    println!("read_edges_from_fmi_file");
    let edges = read_edges_from_fmi_file(&args.graph);

    println!("build graph");
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!("cloning out graph");
    let cloned_graph = graph.clone();

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_witness_search(graph);
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    println!("checking if working");
    let my_graph = cloned_graph.out_graph();
    let my_ch_graph = &contracted_graph.upward_graph;
    for vertex in (0..my_graph.number_of_vertices()).progress() {
        let ch_edges = my_ch_graph.edges(vertex).collect::<HashSet<_>>();

        let mut data = DijkstraDataVec::new(my_ch_graph);
        let mut expanded = VertexExpandedDataBitSet::new(my_ch_graph);
        let mut queue = VertexDistanceQueueBinaryHeap::new();
        let brute_force_ch_edges = get_ch_edges(
            my_ch_graph,
            &mut data,
            &mut expanded,
            &mut queue,
            &contracted_graph.vertex_to_level,
            vertex,
        )
        .into_iter()
        .collect::<HashSet<_>>();

        // The brute force edges are the minimal ammount of edges, it is no problem if
        // there are more ch edges.
        assert!(brute_force_ch_edges.is_subset(&ch_edges));

        // But for all ch edges that are not brute force edges, we need to prove, that
        // they are unnecessary.
        for ch_edge in ch_edges.sub(&brute_force_ch_edges) {
            let mut data = DijkstraDataVec::new(my_ch_graph);
            let mut expanded = VertexExpandedDataBitSet::new(my_ch_graph);
            let mut queue = VertexDistanceQueueBinaryHeap::new();

            dijkstra_one_to_one(
                my_ch_graph,
                &mut data,
                &mut expanded,
                &mut queue,
                vertex,
                ch_edge.head,
            );

            let vertices = data.get_path(ch_edge.head).unwrap().vertices;
            assert!(vertices.iter().any(|&vertex| vertex == ch_edge.head));
        }
    }

    let other_contracted_graph =
        brute_force_contracted_graph(&cloned_graph, &contracted_graph.level_to_vertex);

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());

            let start = Instant::now();
            let ch_distance = ch_one_to_one_wrapped(&other_contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let start = Instant::now();
            let dijkstra_distance =
                dijkstra_one_to_one_wrapped(cloned_graph.out_graph(), source, target);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&ch_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect::<Vec<_>>();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}
