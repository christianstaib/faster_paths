use ch::{
    classical_search::DijkstraPathfinder,
    fmi::{read_fmi_graph, write_tests},
    graph::{FastGraph, GraphLike, WeightedEdge},
    path::{PathDistance, PathQuery},
    pathfinder::ShortestPathFinder,
    types::{Distance, VertexId},
};
use clap::Parser;
use indicatif::ParallelProgressIterator;
use ordered_float::OrderedFloat;
use rand::seq::index::sample;
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph file
    #[arg(short, long)]
    graph: PathBuf,

    /// Output tests file
    #[arg(short, long)]
    tests: PathBuf,

    /// Test count
    #[arg(short = 'n', long)]
    num_tests: usize,
}

fn generate_tests<D: Distance>(
    graph: &FastGraph<WeightedEdge<D>>,
    num_tests: usize,
) -> Vec<PathDistance<D>> {
    let num_vertices = graph.num_vertices();
    let mut rng = rand::rng();

    let queries = (0..num_tests)
        .map(|_| {
            let vertices = sample(&mut rng, num_vertices, 2);

            PathQuery {
                source: VertexId::new(vertices.index(0) as u32),
                target: VertexId::new(vertices.index(1) as u32),
            }
        })
        .collect::<Vec<_>>();

    queries
        .into_par_iter()
        .progress()
        .map_init(
            || DijkstraPathfinder::new(graph),
            |pathfinder, query| PathDistance::new(query, pathfinder.distance(&query)),
        )
        .collect::<Vec<_>>()
}

type DistanceType = OrderedFloat<f32>;

fn main() {
    let args = Args::parse();

    let graph = read_fmi_graph::<DistanceType>(&args.graph).unwrap();

    let tests = generate_tests(&graph, args.num_tests);
    write_tests(&args.tests, &tests).unwrap();

    println!("Wrote {} tests to {:?}.", tests.len(), args.tests);
}
