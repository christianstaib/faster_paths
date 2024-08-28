use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, Vertex,
    },
    utility::{get_paths, level_to_vertex},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file path to the graph in FMI format.
    #[arg(short, long)]
    graph: PathBuf,

    /// Number of searches to perform.
    #[arg(short = 's', long = "searches")]
    number_of_searches: u32,

    /// Number of paths to find per search.
    #[arg(short = 'p', long = "paths")]
    number_of_paths_per_search: u32,

    /// Path to the output file where the vertex to level mapping will be
    /// stored.
    #[arg(short, long)]
    level_to_vertex: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    // Get paths and level_to_vertex
    let paths = get_paths(
        graph.out_graph(),
        args.number_of_searches,
        args.number_of_paths_per_search,
    );
    let level_to_vertex: Vec<Vertex> =
        level_to_vertex(&paths, graph.out_graph().number_of_vertices());

    // Write level_to_vertex to file
    let writer = BufWriter::new(File::create(args.level_to_vertex).unwrap());
    serde_json::to_writer(writer, &level_to_vertex).unwrap();
}
