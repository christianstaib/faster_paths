use std::path::PathBuf;

use clap::Parser;
use faster_paths::graphs::{vec_vec_graph::VecVecGraph, Graph};

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

    println!("Loading graph");
    let graph = VecVecGraph::from_fmi_file(&args.graph);

    println!(
        "graph has {} vertices and {} edges",
        graph.number_of_vertices(),
        graph.number_of_edges()
    );
}
