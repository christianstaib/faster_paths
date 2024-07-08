use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    ch::ch_from_top_down::generate_directed_contracted_graph,
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

    let mut level_to_verticies_map = vec![Vec::new(); vertex_to_level_map.len()];
    for (vertex, &level) in vertex_to_level_map.iter().enumerate() {
        level_to_verticies_map[level as usize].push(vertex as u32);
    }

    let start = Instant::now();

    let contracted_graph = generate_directed_contracted_graph(graph, vertex_to_level_map);

    println!("took {:?}", start.elapsed());

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
