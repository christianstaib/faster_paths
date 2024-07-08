use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    ch::contraction_adaptive_simulated::contract_adaptive_simulated_with_landmarks,
    graphs::graph_factory::GraphFactory,
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

    let contracted_graph = contract_adaptive_simulated_with_landmarks(&graph);
    println!("Generating contracted graph took {:?}", start.elapsed());

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
