use std::{fs::File, io::BufReader};

use clap::Parser;
use indicatif::ProgressIterator;
use osm_test::hl::hub_graph::HubGraph;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    hl_graph: String,
}

fn main() {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.hl_graph).unwrap());
    let hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();

    let mut used = vec![0 as u32; hub_graph.forward_labels.len()];
    hub_graph
        .forward_labels
        .iter()
        .chain(hub_graph.reverse_labels.iter())
        .progress_count(2 * hub_graph.forward_labels.len() as u64)
        .flat_map(|label| &label.entries)
        .for_each(|entry| used[entry.vertex as usize] += 1);

    let all: u64 = used.iter().map(|&n| n as u64).sum();

    used.sort();
    used.reverse();

    let top_256: u64 = used.iter().take(256).map(|&n| n as u64).sum();

    println!("top_256 is {:>2}%", top_256 as f32 / all as f32 * 100.0);
}
