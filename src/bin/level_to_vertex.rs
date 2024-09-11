use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex},
    reading_pathfinder,
    search::ch::contracted_graph::vertex_to_level,
    utility::{
        average_ch_vertex_degree, average_hl_label_size, get_paths, hit_percentage, level_to_vertex,
    },
    FileType,
};

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

    if let Some(hit_percentage_path) = &args.hit_percentage {
        let writer = BufWriter::new(File::create(&hit_percentage_path).unwrap());
        serde_json::to_writer(writer, &hit_percentage(&paths, &level_to_vertex)).unwrap();
    }

    let num_samples = 1_000;
    let average_hl_label_size = average_hl_label_size(
        graph.out_graph(),
        &vertex_to_level(&level_to_vertex),
        num_samples,
    );
    println!(
        "average hl label size will be approximately {:.1}. (averaged over {} out of {} vertices)",
        average_hl_label_size,
        num_samples,
        graph.out_graph().number_of_vertices()
    );

    let average_ch_vertex_degree = average_ch_vertex_degree(
        graph.out_graph(),
        &vertex_to_level(&level_to_vertex),
        num_samples,
    );
    println!(
        "average hl label size will be approximately {:.1}. (averaged over {} out of {} vertices)",
        average_ch_vertex_degree,
        num_samples,
        graph.out_graph().number_of_vertices()
    );
}
