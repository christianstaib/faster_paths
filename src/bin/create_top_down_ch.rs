use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    ch::ch_from_top_down::ch_from_top_down,
    graphs::{
        graph_factory::GraphFactory, graph_functions::generate_vertex_to_level_map, path::Path,
    },
};

/// Creates a hub graph top down.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    paths: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("loading paths");
    let reader = BufReader::new(File::open(&args.paths).unwrap());
    let paths: Vec<Path> = serde_json::from_reader(reader).unwrap();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    let vertex_to_level_map = generate_vertex_to_level_map(paths, graph.number_of_vertices);
    let contracted_graph = ch_from_top_down(graph, vertex_to_level_map);

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
