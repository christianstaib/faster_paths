use ch::{fmi::read_fmi_graph, graph::GraphLike, validation::generate_queries};
use clap::Parser;
use std::{path::PathBuf, time::Instant};

const NUM_QUERIES: usize = 1000;

type DistanceType = u32;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input graph file
    #[arg(short, long)]
    graph: PathBuf,
}

fn main() {
    let args = Args::parse();
    let graph = read_fmi_graph::<DistanceType>(&args.graph).unwrap();

    let mut input_graph = easbar_fast_paths::InputGraph::new();
    for edge in graph.edges() {
        input_graph.add_edge(
            edge.tail.as_usize(),
            edge.head.as_usize(),
            edge.weight as easbar_fast_paths::Weight,
        );
    }
    input_graph.freeze();

    let start = Instant::now();
    let fast_graph = easbar_fast_paths::prepare(&input_graph);
    let contraction_duration = start.elapsed();

    println!("Contracted graph in {:?}.", contraction_duration);

    let num_nodes = fast_graph.get_num_nodes();
    if num_nodes < 2 {
        println!("Graph has fewer than two routable nodes; skipped random queries.");
        return;
    }

    let queries = generate_queries(num_nodes, NUM_QUERIES);
    let mut path_calculator = easbar_fast_paths::create_calculator(&fast_graph);

    let start = Instant::now();
    for query in &queries {
        path_calculator.calc_path(
            &fast_graph,
            query.source.as_usize(),
            query.target.as_usize(),
        );
    }
    let query_duration = start.elapsed();

    println!(
        "Took on average {:?} over {} random queries.",
        query_duration / NUM_QUERIES as u32,
        NUM_QUERIES,
    );
}
