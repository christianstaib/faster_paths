use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::hl::{directed_hub_graph::DirectedHubGraph, HubGraphTrait};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Outfile in .bincode format
    #[arg(short, long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading hub graph");
    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
    let hub_graph: DirectedHubGraph = bincode::deserialize_from(reader).unwrap();

    println!(
        "the hl graph has {} vertices.",
        hub_graph.number_of_vertices()
    );

    let num_label_entries = (0..hub_graph.number_of_vertices())
        .map(|vertex| hub_graph.forward_label(vertex).len() as u64)
        .sum::<u64>();
    println!(
        "the hl forward labels consists of {} entries (avg entries per vertex: {})",
        num_label_entries,
        num_label_entries as f32 / hub_graph.number_of_vertices() as f32
    );
}
