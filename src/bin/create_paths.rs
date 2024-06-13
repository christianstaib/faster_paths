use core::panic;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    graphs::{
        graph_factory::GraphFactory, graph_functions::random_paths, path::PathFinding, Graph,
    },
    hl::directed_hub_graph::DirectedHubGraph,
};
use indicatif::ProgressIterator;
use itertools::Itertools;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    hub_graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    number_of_paths: u32,
    /// Path of .fmi file
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    let path_finder: Box<dyn PathFinding>;

    println!("Loading hub graph");
    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());

    let pathfinder_string = args.hub_graph.to_str().unwrap();
    if pathfinder_string.ends_with(".di.ch.bincode") {
        let contracted_graph: DirectedContractedGraph = bincode::deserialize_from(reader).unwrap();
        path_finder = Box::new(contracted_graph);
    } else if pathfinder_string.ends_with(".di.hl.bincode") {
        let hub_graph: DirectedHubGraph = bincode::deserialize_from(reader).unwrap();
        path_finder = Box::new(hub_graph);
    } else if pathfinder_string.ends_with(".gr") {
        let graph = GraphFactory::from_file(&args.hub_graph);
        path_finder = Box::new(graph);
    } else {
        panic!("cant read file \"{}\"", args.hub_graph.to_str().unwrap());
    }

    println!("Generating random pair paths");
    let paths = random_paths(
        args.number_of_paths,
        path_finder.number_of_vertices() as u32,
        &*path_finder,
    );

    println!("Writing paths to file");
    let mut writer = BufWriter::new(File::create(&args.paths).unwrap());
    for path in paths.iter().progress() {
        writeln!(writer, "{}", path.vertices.iter().join(" ")).unwrap();
    }
    writer.flush().unwrap();
}
