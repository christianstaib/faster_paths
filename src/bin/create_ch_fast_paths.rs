use std::time::Instant;

use clap::Parser;
use fast_paths::InputGraph;
use faster_paths::graphs::graph_factory::GraphFactory;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph_path.as_str());

    let start = Instant::now();
    let mut input_graph = InputGraph::new();
    for edge in graph.all_edges() {
        input_graph.add_edge(edge.tail as usize, edge.head as usize, edge.weight as usize);
    }
    let _ = fast_paths::prepare(&input_graph);
    println!("Generating ch took {:?}", start.elapsed());
}
