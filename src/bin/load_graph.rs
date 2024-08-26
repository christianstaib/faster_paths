use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        ch::{
            contracted_graph::{self, ch_one_to_one_wrapped, ContractedGraph},
            contraction,
        },
        dijkstra::dijkstra_one_to_one_wrapped,
        hl::hub_graph::{get_path_from_overlapp, HubGraph},
        shortcuts::replace_shortcuts_slowly,
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

    println!("Create contracted graph");
    let contracted_graph = ContractedGraph::by_contraction_with_dijkstra_witness_search(&graph);

    if graph.out_graph().is_bidirectional() {
        for vertex in graph.out_graph().vertices() {
            let up = contracted_graph.upward_graph().edges(vertex).collect_vec();
            let down = contracted_graph
                .downward_graph()
                .edges(vertex)
                .collect_vec();

            assert_eq!(up, down);
        }
    }

    println!("brute_force");
    let contracted_graph =
        ContractedGraph::by_brute_force(&graph, contracted_graph.level_to_vertex());

    //  let contracted_graph =
    //      ContractedGraph::by_brute_force(&graph,
    // &contracted_graph.level_to_vertex());

    let hub_graph = HubGraph::by_brute_force(&graph, contracted_graph.vertex_to_level());

    // for &vertex in contracted_graph.level_to_vertex().iter().rev().take(10) {
    //     println!("v:{} {:?}", vertex, hub_graph.forward.get_label(vertex));
    // }

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..graph.out_graph().number_of_vertices());

            let start = Instant::now();
            let mut hl_path = get_path_from_overlapp(
                hub_graph.forward.get_label(source),
                hub_graph.backward.get_label(target),
                &hub_graph.shortcuts,
            );
            // let mut hl_path = ch_one_to_one_wrapped(&contracted_graph, source, target);
            let hl_distance = hl_path.as_ref().map(|path| path.distance);
            let ch_time = start.elapsed().as_secs_f64();

            let distance =
                hl_path.and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
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
