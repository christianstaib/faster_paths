use std::{fs::File, io::BufWriter, time::Instant};

use clap::Parser;
use faster_paths::{
    ch::{contractor::serial_contractor::SerialContractor, preprocessor::Preprocessor},
    graphs::graph_factory::GraphFactory,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of contracted_graph (output)
    #[arg(short, long)]
    ch_graph: String,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph_path.as_str());

    let letters = "E";

    let start = Instant::now();
    let contractor = Box::new(SerialContractor::new(letters));
    let preprocessor = Preprocessor::with_contractor(contractor);
    let contracted_graph = preprocessor.get_ch(&graph);
    println!("Generating ch took {:?}", start.elapsed());

    let writer = BufWriter::new(File::create(args.ch_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
