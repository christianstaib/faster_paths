use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::ch::contracted_graph::vertex_to_level,
    utility::{get_paths, level_to_vertex},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    #[arg(short, long)]
    number_of_searches: u32,
    #[arg(short, long)]
    number_of_paths_per_search: u32,
    #[arg(short, long)]
    vertex_to_level: PathBuf,
}

fn main() {
    let args = Args::parse();

    let edges = read_edges_from_fmi_file(&args.graph);

    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let paths = get_paths(
        graph.out_graph(),
        args.number_of_searches,
        args.number_of_paths_per_search,
    );

    let level_to_vertex = level_to_vertex(&paths, graph.out_graph().number_of_vertices());
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let writer = BufWriter::new(File::create(args.vertex_to_level).unwrap());
    serde_json::to_writer(writer, &vertex_to_level).unwrap();
}
