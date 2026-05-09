use ch::{
    fmi::read_fmi_graph,
    graph::{FastGraph, GraphLike, WeightedEdge},
    types::VertexId,
};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph file
    #[arg(short, long)]
    graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Parse the ChGraph from the file
    let graph: FastGraph<WeightedEdge<u32>> = read_fmi_graph(&args.graph).unwrap();

    for edge in graph.out_edges(VertexId::new(5)) {
        println!("{:?} -> {:?} = {:?}", edge.tail, edge.head, edge.weight);
    }
}
