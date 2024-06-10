use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use ahash::HashMap;
use clap::Parser;
use faster_paths::{
    graphs::{edge::DirectedEdge, graph_functions::random_paths, VertexId},
    hl::{hl_path_finding::HLPathFinder, hub_graph::DirectedHubGraph},
    shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer,
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

    println!("Loading hub graph");
    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
    let (hub_graph, shortcuts): (DirectedHubGraph, HashMap<DirectedEdge, VertexId>) =
        bincode::deserialize_from(reader).unwrap();

    println!("Generating random pair paths");
    let path_finder = HLPathFinder::new(&hub_graph);
    let path_finder = SlowShortcutReplacer::new(&shortcuts, &path_finder);

    let paths = random_paths(
        args.number_of_paths,
        hub_graph.forward_labels.len() as u32,
        &path_finder,
    );

    println!("Writing paths to file");
    let mut writer = BufWriter::new(File::create(&args.paths).unwrap());
    for path in paths.iter().progress() {
        writeln!(writer, "{}", path.vertices.iter().join(" ")).unwrap();
    }
    writer.flush().unwrap();
}
