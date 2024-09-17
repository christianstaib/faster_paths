use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph},
    reading_pathfinder,
    utility::{get_paths, get_progressbar},
    FileType,
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,

    /// Number of seartes.
    #[arg(short, long, default_value = "100000")]
    number_of_searches: u32,

    /// Path to the output file where the vertex to level mapping will be
    /// stored.
    #[arg(short, long)]
    edge_usage: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    // Get paths and level_to_vertex
    let paths = get_paths(
        &*pathfinder,
        &graph.out_graph().non_trivial_vertices(),
        args.number_of_searches,
    );

    let mut edge_map = graph
        .out_graph()
        .all_edges()
        .par_iter()
        .filter(|edge| edge.tail < edge.head)
        .map(|edge| ((edge.tail, edge.head), 0))
        .collect::<HashMap<_, _>>();

    let pb = get_progressbar("creatin edge map", paths.len() as u64);
    for path in paths.into_iter().progress_with(pb) {
        path.into_iter()
            .tuple_windows()
            .into_iter()
            .for_each(|(s, t)| {
                let min = std::cmp::min(s, t);
                let max = std::cmp::max(s, t);
                *edge_map.get_mut(&(min, max)).unwrap() += 1
            });
    }

    let pb = get_progressbar("Writing edge map", edge_map.len() as u64);
    let mut writer = BufWriter::new(File::create(&args.edge_usage.as_path()).unwrap());
    writeln!(writer, "source,tail,hits").unwrap();
    for (edge, hits) in edge_map.into_iter().progress_with(pb) {
        if hits == 0 {
            continue;
        }
        writeln!(writer, "{},{},{}", edge.0, edge.1, hits).unwrap();
    }
    writer.flush().unwrap();
}
