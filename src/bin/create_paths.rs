use core::panic;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    graphs::{graph_functions::random_paths, path::PathFinding},
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
    let number_of_vertices: u32;

    println!("Loading hub graph");
    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());

    if args.hub_graph.to_str().unwrap().ends_with(".di.ch.bincode") {
        let hub_graph: DirectedContractedGraph = bincode::deserialize_from(reader).unwrap();
        number_of_vertices = hub_graph.number_of_vertices();
        path_finder = Box::new(hub_graph);
    } else {
        panic!("cant read file \"{}\"", args.hub_graph.to_str().unwrap());
    }

    println!("Generating random pair paths");
    let paths = random_paths(args.number_of_paths, number_of_vertices, &*path_finder);

    println!("Writing paths to file");
    let mut writer = BufWriter::new(File::create(&args.paths).unwrap());
    for path in paths.iter().progress() {
        writeln!(writer, "{}", path.vertices.iter().join(" ")).unwrap();
    }
    writer.flush().unwrap();
}
