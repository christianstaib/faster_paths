use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::graphs::graph_factory::GraphFactory;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    in_graph: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    out_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading graph");
    let graph = GraphFactory::from_file(&args.in_graph);

    println!("Writing graph");
    let writer = BufWriter::new(File::create(args.out_graph).unwrap());
    bincode::serialize_into(writer, &graph).unwrap();
}
