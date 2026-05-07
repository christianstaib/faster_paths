use ch::{
    fmi::{read_fmi_graph, write_queries},
    graph::{FastGraph, GraphLike, WeightedEdge},
    path::PathQuery,
    types::VertexId,
};
use clap::Parser;
use ordered_float::OrderedFloat;
use rand::seq::index::sample;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Number of queries to generate
    #[arg(short, long)]
    n: usize,

    /// Output query file
    #[arg(short, long)]
    out: PathBuf,
}

fn generate_queries<D>(graph: &FastGraph<WeightedEdge<D>>, n: usize) -> Vec<PathQuery>
where
    D: ch::types::Distance,
{
    let num_vertices = graph.num_vertices();
    let mut rng = rand::rng();

    (0..n)
        .map(|_| {
            let vertices = sample(&mut rng, num_vertices, 2);

            PathQuery {
                source: VertexId::new(vertices.index(0) as u32),
                target: VertexId::new(vertices.index(1) as u32),
            }
        })
        .collect()
}

type DistanceType = OrderedFloat<f32>;

fn main() {
    let args = Args::parse();

    let graph = read_fmi_graph::<DistanceType>(&args.graph).unwrap();
    let queries = generate_queries(&graph, args.n);
    write_queries(&args.out, &queries).unwrap();

    println!("Wrote {} queries to {:?}.", queries.len(), args.out);
}
