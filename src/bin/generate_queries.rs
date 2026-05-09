use ch::{
    fmi::{read_fmi_graph, write_queries},
    graph::GraphLike,
    validation::generate_queries,
};
use clap::Parser;
use ordered_float::OrderedFloat;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph file
    #[arg(short, long)]
    graph: PathBuf,

    /// Query count
    #[arg(short, long)]
    n: usize,

    /// Output query file
    #[arg(short, long)]
    out: PathBuf,
}

type DistanceType = OrderedFloat<f32>;

fn main() {
    let args = Args::parse();

    let graph = read_fmi_graph::<DistanceType>(&args.graph).unwrap();
    let queries = generate_queries(graph.num_vertices(), args.n);
    write_queries(&args.out, &queries).unwrap();

    println!("Wrote {} queries to {:?}.", queries.len(), args.out);
}
