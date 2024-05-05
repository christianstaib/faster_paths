use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use ahash::HashMap;
use clap::Parser;
use faster_paths::{
    graphs::{edge::DirectedEdge, graph_functions::random_paths, VertexId},
    hl::{hl_path_finding::HLPathFinder, hub_graph::HubGraph},
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
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading graph");
    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
    let (hub_graph, shortcuts): (HubGraph, HashMap<DirectedEdge, VertexId>) =
        bincode::deserialize_from(reader).unwrap();

    println!(
        "avg label size is {}",
        hub_graph
            .labels
            .iter()
            .map(|l| l.entries.len())
            .sum::<usize>()
            / hub_graph.labels.len()
    );

    println!("11111");
    println!(
        "{:?}",
        hub_graph.labels[11_111]
            .entries
            .iter()
            .map(|entry| entry.vertex)
            .collect_vec()
    );

    println!("Generating paths");
    let path_finder = HLPathFinder::new(&hub_graph);
    let path_finder = SlowShortcutReplacer::new(&shortcuts, &path_finder);

    let paths = random_paths(100_000, hub_graph.labels.len() as u32, &path_finder);

    println!("Writing paths to file");
    let mut writer = BufWriter::new(File::create(&args.paths).unwrap());
    for path in paths.iter().progress() {
        writeln!(writer, "{}", path.vertices.iter().join(" ")).unwrap();
    }
    writer.flush().unwrap();
}
