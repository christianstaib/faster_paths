use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex},
    search::PathFinding,
    utility::{get_progressbar, write_json_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Path of test cases
    #[arg(short, long)]
    num_paths: u32,

    /// Path of test cases
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let pb = get_progressbar("Getting paths", args.num_paths as u64);

    let vertices = graph.out_graph().vertices().collect_vec();
    let paths = (0..)
        .par_bridge()
        .map_init(
            || thread_rng(),
            |rng, _| {
                let (source, target): (Vertex, Vertex) = vertices
                    .choose_multiple(rng, 2)
                    .cloned()
                    .collect_tuple()
                    .unwrap();

                graph.shortest_path(source, target)
            },
        )
        .flatten()
        .take_any(args.num_paths as usize)
        .progress_with(pb)
        .collect::<Vec<_>>();

    write_json_with_spinnner("paths", &args.paths, &paths);
}
