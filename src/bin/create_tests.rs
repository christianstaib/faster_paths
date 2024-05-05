use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    classical_search::dijkstra::Dijkstra,
    dijkstra_data::DijkstraData,
    graphs::{
        graph_factory::GraphFactory,
        path::{ShortestPathRequest, ShortestPathTestCase},
        Graph,
    },
};
use indicatif::ProgressIterator;
use rand::Rng;
use rayon::prelude::*;

/// Generates `number_of_tests` many random pair test cases for the graph
/// specified at `graph`. The test cases will be saved at `random_pairs`. For
/// larger `number_of_tests` and complex `graph`s this may take a while.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Graph in `.fmi` of `.gr` format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path where the test cases will be saved
    #[arg(short, long)]
    random_pairs: PathBuf,
    /// TODO
    #[arg(short, long)]
    dijkstra_rank_pairs: PathBuf,
    /// Number of tests to be generated
    #[arg(short, long)]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_file(&args.graph);
    let dijkstra = Dijkstra::new(&graph);

    println!("generating random pair test");
    let start = Instant::now();
    let random_pairs: Vec<_> = (0..args.number_of_tests)
        .progress()
        .par_bridge()
        .map_init(
            rand::thread_rng, // get the thread-local RNG
            |rng, _| {
                // guarantee that source != tatget.
                let source = rng.gen_range(0..graph.number_of_vertices());
                let mut target = rng.gen_range(0..graph.number_of_vertices() - 1);
                if target >= source {
                    target += 1;
                }

                let request = ShortestPathRequest::new(source, target).unwrap();

                let data = dijkstra.get_data(request.source(), request.target());
                let path = data.get_path(target);

                let mut weight = None;
                if let Some(path) = path {
                    weight = Some(path.weight);
                }

                ShortestPathTestCase {
                    request,
                    weight,
                    dijkstra_rank: data.dijkstra_rank(),
                }
            },
        )
        .collect();
    println!("took {:?}", start.elapsed());

    let mut writer = BufWriter::new(File::create(&args.random_pairs).unwrap());
    serde_json::to_writer_pretty(&mut writer, &random_pairs).unwrap();
    writer.flush().unwrap();
}
