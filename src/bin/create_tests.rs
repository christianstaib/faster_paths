use std::{fs::File, io::BufWriter, io::Write};

use clap::Parser;
use faster_paths::{
    graphs::fast_graph::FastGraph,
    graphs::graph_factory::GraphFactory,
    graphs::path::{PathFinding, ShortestPathRequest, ShortestPathValidation},
    simple_algorithms::dijkstra::Dijkstra,
};
use indicatif::ProgressIterator;
use rand::Rng;
use rayon::iter::{ParallelBridge, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
    /// Number of tests to be run
    #[arg(short, long)]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph_path.as_str());
    let graph = FastGraph::from_graph(&graph);
    let dijkstra = Dijkstra::new(&graph);

    let routes: Vec<_> = (0..args.number_of_tests)
        .progress()
        .par_bridge()
        .map_init(
            || rand::thread_rng(), // get the thread-local RNG
            |rng, _| {
                // guarantee that source != tatget.
                let source = rng.gen_range(0..graph.number_of_vertices()) as u32;
                let mut target = rng.gen_range(0..(graph.number_of_vertices()) - 1) as u32;
                if target >= source {
                    target += 1;
                }

                let request = ShortestPathRequest::new(source, target).unwrap();

                let response = dijkstra.get_shortest_path(&request);
                let mut cost = None;
                if let Some(route) = response {
                    cost = Some(route.weight);
                }

                ShortestPathValidation {
                    request,
                    weight: cost,
                }
            },
        )
        .collect();

    let mut writer = BufWriter::new(File::create(args.tests_path.as_str()).unwrap());
    serde_json::to_writer_pretty(&mut writer, &routes).unwrap();
    writer.flush().unwrap();
}
