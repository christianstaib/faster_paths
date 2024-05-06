use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use ahash::HashMap;
use clap::Parser;
use faster_paths::{
    ch::contracted_graph::DirectedContractedGraph,
    graphs::{edge::DirectedEdge, VertexId},
    hl::hl_from_ch::directed_hub_graph_from_directed_contracted_graph,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    contracted_graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading contracted graph");
    let reader = BufReader::new(File::open(args.contracted_graph).unwrap());
    let (contracted_graph, shortcuts): (DirectedContractedGraph, HashMap<DirectedEdge, VertexId>) =
        bincode::deserialize_from(reader).unwrap();

    println!("Start hub graph geneation");
    let start = Instant::now();
    let hub_graph = directed_hub_graph_from_directed_contracted_graph(&contracted_graph);
    println!("Generating hub graph took {:?}", start.elapsed());

    println!("Writing hub graph to file");
    let writer = BufWriter::new(File::create(args.hub_graph).unwrap());
    bincode::serialize_into(writer, &(&hub_graph, &shortcuts)).unwrap();
}
