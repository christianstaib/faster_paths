use std::{fs::File, io::BufWriter, time::Instant};

use clap::Parser;
use faster_paths::{
    ch::{
        contractor::Contractor,
        graph_cleaner::{remove_edge_to_self, removing_double_edges},
    },
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

    let mut graph = GraphFactory::from_gr_file(args.graph_path.as_str());
    removing_double_edges(&mut graph);
    remove_edge_to_self(&mut graph);

    let start = Instant::now();
    let contracted_graph = Contractor::get_contracted_graph(&graph);
    println!("Generating ch took {:?}", start.elapsed());

    let writer = BufWriter::new(File::create(args.ch_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
