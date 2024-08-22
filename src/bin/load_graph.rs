use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, WeightedEdge,
    },
    search::{
        ch::{
            brute_force::get_ch_edges,
            contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
            contraction::contraction_with_witness_search,
        },
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::VertexDistanceQueueBinaryHeap,
            vertex_expanded_data::VertexExpandedDataBitSet,
        },
        dijkstra::{dijkstra_one_to_all_wraped, dijkstra_one_to_one_wrapped},
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

    println!(
        "{:?}",
        dijkstra_one_to_all_wraped(graph.out_graph(), 3474)
            .get_path(3548)
            .unwrap()
    );

    println!("cloning out graph");
    let cloned_graph = graph.clone();

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_witness_search(graph);
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    // println!("checking if working");
    // for vertex in (0..cloned_graph.out_graph().number_of_vertices()).progress() {
    //     // let vertex = 3474;
    //     let mut ch_edges =
    // contracted_graph.upward_graph.edges(vertex).collect_vec();     ch_edges.
    // sort_by_key(|edge| edge.head);

    //     let mut data = DijkstraDataVec::new(cloned_graph.out_graph());
    //     let mut expanded =
    // VertexExpandedDataBitSet::new(cloned_graph.out_graph());     let mut
    // queue = VertexDistanceQueueBinaryHeap::new();     let mut
    // brute_force_ch_edges = get_ch_edges(         cloned_graph.out_graph(),
    //         &mut data,
    //         &mut expanded,
    //         &mut queue,
    //         &contracted_graph.vertex_to_level,
    //         vertex,
    //     );
    //     brute_force_ch_edges.sort_by_key(|edge| edge.head);

    //     assert_eq!(ch_edges, brute_force_ch_edges, "{}", vertex);
    // }

    let upward_edges = create_ch_edges(cloned_graph.out_graph(), &contracted_graph.vertex_to_level);
    let downard_edges = create_ch_edges(cloned_graph.in_graph(), &contracted_graph.vertex_to_level);
    let upward_graph = VecVecGraph::from_edges(&upward_edges);
    let downward_graph = VecVecGraph::from_edges(&downard_edges);

    let other_contracted_graph = ContractedGraph {
        upward_graph,
        downward_graph,
        level_to_vertex,
        vertex_to_level: contracted_graph.vertex_to_level.clone(),
    };

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());

            let start = Instant::now();
            let ch_distance = ch_one_to_one_wrapped(&contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let start = Instant::now();
            let dijkstra_distance =
                dijkstra_one_to_one_wrapped(cloned_graph.out_graph(), source, target);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&ch_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect_vec();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}

fn create_ch_edges(graph: &dyn Graph, vertex_to_level: &Vec<u32>) -> Vec<WeightedEdge> {
    (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map_init(
            || {
                (
                    DijkstraDataVec::new(graph),
                    VertexExpandedDataBitSet::new(graph),
                    VertexDistanceQueueBinaryHeap::new(),
                )
            },
            |(data, expanded, queue), vertex| {
                get_ch_edges(graph, data, expanded, queue, vertex_to_level, vertex)
            },
        )
        .flatten()
        .collect::<Vec<_>>()
}
