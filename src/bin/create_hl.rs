use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    ch::contracted_graph::DirectedContractedGraph,
    hl::{
        hl_from_ch::directed_hub_graph_from_directed_contracted_graph,
        hub_graph_investigator::get_avg_label_size,
    },
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    ch_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    hl_graph: String,
}

fn main() {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.ch_graph).unwrap());
    let contracted_graph: DirectedContractedGraph = bincode::deserialize_from(reader).unwrap();

    // // optimize levels
    // println!("{}", contracted_graph.levels.len());
    // let mut new_levels = vec![Vec::new()];
    // let mut current_neighbors = HashSet::new();

    // let vertices: Vec<_> =
    // contracted_graph.levels.iter().flatten().cloned().collect();

    // for &vertex in vertices.iter().progress() {
    //     if current_neighbors.contains(&vertex) {
    //         new_levels.push(Vec::new());
    //         current_neighbors.clear();
    //     }
    //     new_levels.last_mut().unwrap().push(vertex);
    //     current_neighbors.extend(contracted_graph.graph.open_neighborhood(vertex,
    // 1)); }

    // contracted_graph.levels = new_levels;
    println!("{}", contracted_graph.levels.len());

    let start = Instant::now();
    let hub_graph = directed_hub_graph_from_directed_contracted_graph(&contracted_graph);
    println!("Generating hl took {:?}", start.elapsed());

    let writer = BufWriter::new(File::create(args.hl_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();

    println!("average label size is {}", get_avg_label_size(&hub_graph));
}
