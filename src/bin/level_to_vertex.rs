use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex},
    reading_pathfinder,
    search::ch::contracted_graph::vertex_to_level,
    utility::{average_label_size, get_paths, get_progressbar, level_to_vertex},
    FileType,
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
    level_to_vertex: PathBuf,

    #[arg(short, long)]
    hit_percentage: Option<PathBuf>,
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
    let level_to_vertex: Vec<Vertex> = level_to_vertex(&paths, pathfinder.number_of_vertices());

    // Write level_to_vertex to file
    let writer = BufWriter::new(File::create(args.level_to_vertex).unwrap());
    serde_json::to_writer(writer, &level_to_vertex).unwrap();

    if let Some(hit_percentage) = &args.hit_percentage {
        let writer = BufWriter::new(File::create(&hit_percentage).unwrap());
        serde_json::to_writer(writer, &hitable(&paths, &level_to_vertex)).unwrap();
    }

    let average_label_size =
        average_label_size(graph.out_graph(), &vertex_to_level(&level_to_vertex), 1_000);
    println!("average label size is {:.1}", average_label_size);
}

pub fn hitable(paths: &Vec<Vec<Vertex>>, level_to_vertex: &Vec<Vertex>) -> Vec<f32> {
    let mut hit_percentage = Vec::new();
    let mut active_paths = paths.iter().collect_vec();

    let pb = get_progressbar("Getting hit percentages", level_to_vertex.len() as u64);

    // highest level first
    for &hitting_vertex in level_to_vertex.iter().rev().progress_with(pb) {
        active_paths = active_paths
            .into_par_iter()
            .filter(|path| !path.contains(&hitting_vertex))
            .collect();

        hit_percentage.push((paths.len() - active_paths.len()) as f32 / paths.len() as f32)
    }

    hit_percentage
}
