use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::search::ch::contracted_graph::ContractedGraph;
use itertools::Itertools;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    degrees_out: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Read contracted_graph
    let reader = BufReader::new(File::open(&args.contracted_graph).unwrap());
    let contracted_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();

    println!(
        "vertices:{} edges:{}",
        contracted_graph.upward_graph().number_of_vertices(),
        contracted_graph.upward_graph().number_of_edges()
    );

    let out_degrees = contracted_graph
        .upward_graph()
        .vertices()
        .map(|vertex| contracted_graph.upward_graph().edges(vertex).len())
        .collect_vec();
    let in_degrees = contracted_graph
        .upward_graph()
        .vertices()
        .map(|vertex| contracted_graph.upward_graph().edges(vertex).len())
        .collect_vec();
    let writer = BufWriter::new(File::create(&args.degrees_out).unwrap());
    serde_json::to_writer(writer, &out_degrees).unwrap();

    let non_trivial_vertices = contracted_graph
        .upward_graph()
        .vertices()
        .filter(|&vertex| out_degrees[vertex as usize] == 0 && in_degrees[vertex as usize] == 0)
        .count();
    println!("non trivial vertices: {}", non_trivial_vertices);
}
