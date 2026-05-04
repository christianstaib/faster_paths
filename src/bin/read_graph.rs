use ch::{
    flattened_nested::FlattenedNested,
    fmi_helper::read_fmi_graph,
    types::{Distance, VertexId},
};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Parse the ChGraph from the file
    let graph: ch::flattened_nested::FlattenedNested<ch::edge::Edge> =
        read_fmi_graph(&args.graph_in).unwrap();

    for edge in graph.nested(5) {
        println!(
            "{:?} -> {:?} = {:?}",
            edge.tail(),
            edge.target(),
            edge.weight()
        );
    }

    let path = vec![VertexId::new(5), VertexId::new(14595642)];

    println!("{:?}", sum_path(&path, &graph));
}

pub fn sum_path(path: &[VertexId], graph: &FlattenedNested<ch::edge::Edge>) -> Option<Distance> {
    let mut sum = Distance::ZERO;

    for window in path.windows(2) {
        let tail = window[0];
        let head = window[1];

        let edges = graph.nested(tail.as_usize());

        if let Some(index) = edges.binary_search_by_key(&head, |edge| edge.target()).ok() {
            sum = sum + edges[index].weight();
        } else {
            return None;
        }
    }

    Some(sum)
}
