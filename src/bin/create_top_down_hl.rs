use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{generate_hiting_set_order, generate_hiting_set_order_with_hub_labels},
    },
    hl::top_down_hl::generate_hub_graph,
};

/// Creates a hub graph top down.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    let number_of_random_pairs = 4_000;
    let order = generate_hiting_set_order_with_hub_labels(number_of_random_pairs, &graph);

    println!("Generating hub graph");
    let start = Instant::now();
    let hub_graph_and_shortcuts = generate_hub_graph(&graph, &order);
    println!("Generating all labels took {:?}", start.elapsed());

    println!("Saving hub graph as json");
    let writer = BufWriter::new(File::create(&args.hub_graph).unwrap());
    serde_json::to_writer(writer, &hub_graph_and_shortcuts).unwrap();
}
