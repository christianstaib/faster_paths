use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::Vertex,
    reading_pathfinder,
    search::PathFinding,
    utility::{get_progressbar, write_json_with_spinnner},
    FileType,
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,

    /// Path of test cases
    #[arg(short, long)]
    num_paths: u32,

    /// Path of test cases
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    let pb = get_progressbar("Getting paths", args.num_paths as u64);

    let vertices = (0..pathfinder.number_of_vertices()).collect_vec();
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

                pathfinder.shortest_path(source, target)
            },
        )
        .flatten()
        .take_any(args.num_paths as usize)
        .progress_with(pb)
        .collect::<Vec<_>>();

    write_json_with_spinnner("paths", &args.paths, &paths);
}
