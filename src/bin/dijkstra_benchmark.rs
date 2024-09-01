use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph},
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataVec},
        },
        dijkstra::dijkstra_one_to_one,
        PathFinding,
    },
};
use rand::{thread_rng, Rng};

/// Does a single threaded benchmark.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    in_file: PathBuf,
    /// Number of benchmarks to be run.
    #[arg(short, long)]
    number_of_benchmarks: u32,
}

#[derive(Debug, ValueEnum, Clone)]
enum FileType {
    CH,
    HL,
    FMI,
}

fn main() {
    let args = Args::parse();

    let graph: ReversibleGraph<VecVecGraph> = {
        let reader = BufReader::new(File::open(&args.in_file).unwrap());
        bincode::deserialize_from(reader).unwrap()
    };

    let out_graph = graph.out_graph();

    let mut durations = Vec::new();

    let mut data = DijkstraDataVec::new(out_graph);
    let mut expanded = VertexExpandedDataVec::new(out_graph);
    let mut queue = VertexDistanceQueueBinaryHeap::new();

    let mut rng = thread_rng();
    (0..args.number_of_benchmarks).for_each(|_| {
        let source = rng.gen_range(0..graph.number_of_vertices());
        let target = rng.gen_range(0..graph.number_of_vertices());

        let start = Instant::now();
        dijkstra_one_to_one(
            out_graph,
            &mut data,
            &mut expanded,
            &mut queue,
            source,
            target,
        );

        data.clear();
        expanded.clear();
        queue.clear();
        durations.push(start.elapsed());

        println!(
            "Average duration {:?}",
            durations.iter().sum::<Duration>() / durations.len() as u32
        );
    })
}
