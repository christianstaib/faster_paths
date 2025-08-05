use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    degree: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let mut writer = BufWriter::new(File::create(args.degree.as_path()).unwrap());
    for vertex in graph.out_graph().vertices() {
        writeln!(writer, "{}", graph.out_graph().edges(vertex).len()).unwrap();
    }
    writer.flush().unwrap();
}
