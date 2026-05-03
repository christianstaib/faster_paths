use ch::ch::contraction_hierarchy::ContractionHierarchy;
use ch::ch::edge::Edge;
use ch::ch::pathfinder::Pathfinder;
use ch::flattened_nested::FlattenedNested;
use ch::search_state::hash_search_state::HashSearchState;
use ch::types::{Distance, VertexId};
use clap::Parser;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,
    ///
    /// Infile in .fmi format
    #[arg(short, long)]
    test_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Open and read the graph file
    let reader = BufReader::new(File::open(&args.graph_in).unwrap());

    // Parse the ChGraph from the file
    let graph = from_reader(reader).unwrap();

    let mut pathfiner = Pathfinder::new(
        &graph,
        BinaryHeap::new(),
        HashSearchState::new(),
        HashSearchState::new(),
    );

    let mut tests = Vec::new();
    {
        let reader_test = BufReader::new(File::open(&args.test_in).unwrap());
        let mut test_lines = reader_test.lines().flatten();
        test_lines.next();
        while let Some(line) = test_lines.next() {
            let mut parts = line.split_whitespace();

            let source = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
            let target = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
            let true_distance: Option<Distance> =
                parts.next().unwrap().parse().ok().map(|x| Distance::new(x));

            tests.push((source, target, true_distance));
        }
    }

    let start = Instant::now();
    let correct = tests.iter().all(|&(source, target, true_distance)| {
        pathfiner
            .search(source, target)
            .map(|(distance, _vertex)| distance)
            == true_distance
    });
    let whole_duration = start.elapsed();

    println!(
        "Took {:?} on average. All correct? {:?}",
        whole_duration / tests.len() as u32,
        correct
    );
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
