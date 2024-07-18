use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_from_top_down::ch_from_top_down, contraction_with_fixed_order::contract_with_fixed_order,
    },
    graphs::{
        graph_factory::GraphFactory, graph_functions::generate_vertex_to_level_map, path::Path,
        VertexId,
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
    // let contracted_graph = ch_from_top_down(graph, vertex_to_level_map);

    let max_level = *vertex_to_level_map.iter().max().unwrap();
    let mut level_to_vertices_map = vec![Vec::new(); max_level as usize + 1];

    for (vertex, &level) in vertex_to_level_map.iter().enumerate() {
        level_to_vertices_map[level as usize].push(vertex as VertexId);
    }

    let contracted_graph = contract_with_fixed_order(&graph, &level_to_vertices_map);

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
