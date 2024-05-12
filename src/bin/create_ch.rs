use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    ch::contraction_non_adaptive::contract_non_adaptive,
    graphs::{
        graph_factory::GraphFactory, graph_functions::generate_hiting_set_order_with_hub_labels,
    },
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    println!("Starting contracted graph generation");
    let start = Instant::now();

    let number_of_random_pairs = 4_000;
    let mut order = generate_hiting_set_order_with_hub_labels(number_of_random_pairs, &graph);
    let order_copy = order.clone();
    order.sort_unstable_by_key(|v| order_copy.iter().position(|vv| vv == v).unwrap());

    let contracted_graph = contract_non_adaptive(&graph, &order);

    // let contracted_graph = contract_adaptive_simulated_with_witness(&graph);
    println!("Generating contracted graph took {:?}", start.elapsed());

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    serde_json::to_writer(writer, &contracted_graph).unwrap();
}
