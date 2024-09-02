use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::graphs::{
    read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph,
};
use itertools::Itertools;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    degrees_out: PathBuf,
}

fn main() {
    let args = Args::parse();

    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!(
        "vertices:{} edges:{}",
        graph.out_graph().number_of_vertices(),
        graph.out_graph().number_of_edges()
    );
    println!(
        "Is graph bidirectional? {}",
        graph.out_graph().is_bidirectional()
    );

    let out_degrees = graph
        .out_graph()
        .vertices()
        .map(|vertex| graph.out_graph().edges(vertex).len())
        .collect_vec();
    let in_degrees = graph
        .in_graph()
        .vertices()
        .map(|vertex| graph.out_graph().edges(vertex).len())
        .collect_vec();
    let writer = BufWriter::new(File::create(&args.degrees_out).unwrap());
    serde_json::to_writer(writer, &out_degrees).unwrap();

    let non_trivial_vertices = graph
        .out_graph()
        .vertices()
        .filter(|&vertex| out_degrees[vertex as usize] == 0 && in_degrees[vertex as usize] == 0)
        .count();
    println!("non trivial vertices: {}", non_trivial_vertices);
}
