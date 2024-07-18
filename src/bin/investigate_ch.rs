use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    graphs::{path::PathFinding, Graph},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Outfile in .bincode format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading contracted graph");
    let reader = BufReader::new(File::open(&args.contracted_graph).unwrap());
    let contracted_graph: DirectedContractedGraph = bincode::deserialize_from(reader).unwrap();

    println!(
        "the ch graph has {} vertices.",
        contracted_graph.number_of_vertices()
    );

    println!(
        "the upward graph has {} edges (avg edges per vertex: {})",
        contracted_graph.upward_graph.number_of_edges(),
        contracted_graph.upward_graph.number_of_edges() as f32
            / contracted_graph.number_of_vertices() as f32,
    );

    println!(
        "the downward graph has {} edges  (avg edges per vertex: {})",
        contracted_graph.downward_graph.number_of_edges(),
        contracted_graph.downward_graph.number_of_edges() as f32
            / contracted_graph.number_of_vertices() as f32
    );

    println!(
        "the ch graph contrains {} shortcuts",
        contracted_graph.shortcuts.len()
    );
}
