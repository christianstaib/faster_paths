use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::graphs::{
    read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
};

/// Reading a .bincode file is way faster than a .fmi file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short = 'f', long)]
    graph_fmi: PathBuf,
    /// Infile in .fmi format
    #[arg(short = 'b', long)]
    graph_bincode: PathBuf,
}

fn main() {
    let args = Args::parse();

    let start = Instant::now();
    let edges = read_edges_from_fmi_file(&args.graph_fmi);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);
    println!("Reading fmi graph took {:?}", start.elapsed());

    let start = Instant::now();
    let writer = BufWriter::new(File::create(&args.graph_bincode).unwrap());
    bincode::serialize_into(writer, &graph).unwrap();
    println!("Writing bincode took {:?}", start.elapsed());
}
