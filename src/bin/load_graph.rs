use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, read_edges_from_gr_file, reversible_graph::ReversibleGraph,
        vec_vec_graph::VecVecGraph, Distance, Edge, Graph, Vertex,
    },
    search::{
        ch::contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
        collections::dijkstra_data::Path,
        dijkstra::dijkstra_one_to_one_wrapped,
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;
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

    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!(
        "Is this graph bidirectional? {}",
        graph.out_graph().is_bidirectional()
    );

    println!("Create contracted graph");
    let contracted_graph = ContractedGraph::by_contraction_with_dijkstra_witness_search(&graph);

    // println!("brute_force");
    // let contracted_graph =
    //     ContractedGraph::by_brute_force(&graph,
    // contracted_graph.level_to_vertex());

    let mut rng = thread_rng();
    let speedup = (0..10_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..graph.out_graph().number_of_vertices());
            // let source = 965726;
            // let target = 646694;

            // println!("{} {}", source, target);

            let start = Instant::now();
            let hl_path = ch_one_to_one_wrapped(&contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let hl_distance = hl_path.as_ref().map(|path| path.distance);

            let distance =
                hl_path.and_then(|path| get_path_distance(graph.out_graph(), &path.vertices));
            assert_eq!(distance, hl_distance);

            let start = Instant::now();
            let dijkstra_distance = dijkstra_one_to_one_wrapped(graph.out_graph(), source, target)
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

fn get_path_distance(graph: &dyn Graph, path: &Vec<Vertex>) -> Option<Distance> {
    path.iter()
        .tuple_windows()
        .map(|(&tail, &head)| graph.get_weight(&Edge { tail, head }))
        .sum()
}
