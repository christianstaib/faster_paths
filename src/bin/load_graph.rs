use std::{collections::HashSet, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        ch::{
            contracted_graph::{self, ContractedGraph},
            contraction::contraction_with_witness_search,
        },
        dijkstra::dijkstra_one_to_one_wrapped,
        hl::hub_graph::{overlapp, HubGraph},
    },
};
use indicatif::ProgressIterator;
use rand::{thread_rng, Rng};

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
    // println!("create graph");
    let contracted_graph = ContractedGraph::from_edges(edges, &level_to_vertex);

    println!("brute_force");
    // let hub_graph_brute_force =
    //     HubGraph::by_brute_force(&cloned_graph,
    // contracted_graph.vertex_to_level()); let mut hub_graph_merging =
    // HubGraph::by_merging(&contracted_graph);

    // let hub_graph = hub_graph_brute_force;

    // let sum_label_len = (0..cloned_graph.out_graph().number_of_vertices())
    //     .map(|vertex| hub_graph.forward.get_label(vertex).len())
    //     .sum::<usize>();
    // println!(
    //     "average label len {}",
    //     sum_label_len as f64 / cloned_graph.out_graph().number_of_vertices() as
    // f64 );

    let contracted_graph =
        ContractedGraph::by_brute_force(&cloned_graph, contracted_graph.level_to_vertex());

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());

            let start = Instant::now();
            // let hl_distance = overlapp(forward_label, backward_label).map(|(distance, _)|
            // distance);
            let hl_distance = contracted_graph.shortest_path_distance(source, target); // overlapp(forward_label, backward_label).map(|(distance, _)| distance);
            let ch_time = start.elapsed().as_secs_f64();

            let start = Instant::now();
            let dijkstra_distance =
                dijkstra_one_to_one_wrapped(cloned_graph.out_graph(), source, target)
                    .map(|path| path.distance);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&hl_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect::<Vec<_>>();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}
