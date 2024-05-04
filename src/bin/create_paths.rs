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

    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
    let (hub_graph, shortcuts): (HubGraph, HashMap<DirectedEdge, VertexId>) =
        bincode::deserialize_from(reader).unwrap();

    let path_finder = HLPathFinder::new(&hub_graph);
    let path_finder = SlowShortcutReplacer::new(&shortcuts, &path_finder);

    let paths = random_paths(100_000, hub_graph.labels.len() as u32, &path_finder);

    let mut writer = BufWriter::new(File::create(&args.paths).unwrap());
    for path in paths {
        writeln!(writer, "{}", path.vertices.iter().join(" ")).unwrap();
    }
}
