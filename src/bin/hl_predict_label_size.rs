use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Vertex,
    },
    search::ch::contracted_graph::vertex_to_level,
    utility::average_label_size,
};

// Predict average label size by brute forcing a number of labels.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Where level_to_vertex list shall be stored.
    #[arg(short, long)]
    level_to_vertex: PathBuf,

    /// Number of labels to calculate
    #[arg(short, long)]
    num_labels: u32,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);
    //
    // Read vertex_to_level
    let reader = BufReader::new(File::open(&args.level_to_vertex).unwrap());
    let level_to_vertex: Vec<Vertex> = serde_json::from_reader(reader).unwrap();
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let average_label_size =
        average_label_size(graph.out_graph(), &vertex_to_level, args.num_labels);
    println!("average label size is {:.1}", average_label_size);
}
