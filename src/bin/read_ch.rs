use ch::ch::contraction_hierarchy::ContractionHierarchy;
use ch::ch::edge::Edge;
use ch::flattened_nested::FlattenedNested;
use ch::types::{Distance, VertexId};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
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

    // Open and read the graph file
    let reader = BufReader::new(File::open(&args.graph_in).unwrap());

    // Parse the ChGraph from the file
    let graph = from_reader(reader).unwrap();

    // Print all edges in up_graph starting from node 3
    let start_node = 3;
    let edges = graph.up_graph().nested(start_node);

    println!("Edges in up_graph starting from node {}:", start_node);
    for edge in edges {
        println!(
            "  {:?} -> {:?}, weight: {:?}, skipped: {:?}",
            edge.tail(),
            edge.head(),
            edge.weight(),
            edge.skiped()
        );
    }
}

fn parse_edge(line: &str) -> Option<Edge> {
    let mut parts = line.split_whitespace();

    let tail = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
    let head = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
    let weight = Distance::new(parts.next()?.parse().ok()?);
    let skipped = parts.next()?.parse().ok().map(|x| VertexId::new(x));

    Some(Edge::new(tail, head, weight, skipped))
}

fn read_edges(lines: &mut impl Iterator<Item = String>, count: usize) -> Option<Vec<Vec<Edge>>> {
    let mut graph = Vec::new();

    for _ in 0..count {
        let edge = parse_edge(&lines.next()?)?;
        let tail = edge.tail().as_usize();

        if graph.len() <= tail {
            graph.resize_with(tail + 1, Vec::new);
        }

        graph[tail].push(edge);
    }

    Some(graph)
}

pub fn from_reader<R: Read>(reader: R) -> Option<ContractionHierarchy> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_up_edges = lines.next()?.parse().ok()?;
    let num_down_edges = lines.next()?.parse().ok()?;

    Some(ContractionHierarchy::new(
        FlattenedNested::new(read_edges(&mut lines, num_up_edges)?),
        FlattenedNested::new(read_edges(&mut lines, num_down_edges)?),
    ))
}
