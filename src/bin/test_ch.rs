use ch::contraction_hierachy::ContractionHierarchyPathfinder;
use ch::flattened_nested::FlattenedNested;
use ch::fmi::read_fmi_ch;
use ch::fmi::read_fmi_graph;
use ch::fmi::read_tests;
use ch::graph::Edge;
use ch::graph::EdgeLike;
use ch::graph::GraphLike;
use ch::graph::WeightedEdge;
use ch::path::{Path, PathDistance, PathQuery};
use ch::pathfinder::ShortestPathFinder;
use ch::types::{Distance, VertexId};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// CH graph in .fmi format
    #[arg(short, long)]
    ch_in: PathBuf,

    /// CH graph in .fmi format
    #[arg(short, long)]
    graph_in: PathBuf,

    /// Test queries in .txt format
    #[arg(short, long)]
    test_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    let ch = read_fmi_ch(&args.ch_in).unwrap();
    let graph = read_fmi_graph(&args.graph_in).unwrap();
    let tests = read_tests(&args.test_in).unwrap();

    let mut pathfinder = ContractionHierarchyPathfinder::new(&ch);

    let failures = tests
        .iter()
        .filter_map(|test| {
            let path = pathfinder.path(test.query());
            validate_path(&graph, test, &path).err()
        })
        .inspect(|message| eprintln!("{message}"))
        .count();

    if failures > 0 {
        eprintln!("{failures} of {} paths failed.", tests.len());
        std::process::exit(1);
    }

    println!("All {} paths correct.", tests.len());
}

/// Sum up the edge weights of `path` in `graph`. If an edge is not found, it is returns as Err.
fn sum_edge_weights<G: GraphLike>(graph: &G, path: &[VertexId]) -> Result<Distance, Edge>
where
    <G as GraphLike>::Edge: Ord,
{
    path.windows(2)
        .try_fold(Distance::ZERO, |summed_distance, potential_edge| {
            let tail = potential_edge[0];
            let head = potential_edge[1];

            let weight = graph
                .out_edges(tail)
                .iter()
                .filter(|edge| edge.head() == head)
                .map(|edge| edge.weight())
                .min()
                .ok_or(Edge { tail, head })?;

            Ok(summed_distance + weight)
        })
}

fn validate_distance(test: &PathDistance, actual: &Option<Distance>) -> Result<(), String> {
    match (&test.distance(), actual) {
        (None, None) => Ok(()),

        (None, Some(actual_distance)) => Err(format!(
            "{:?}. Expected no path, but found one with distance {:?}.",
            test.query(),
            actual_distance
        )),

        (Some(expected_distance), None) => Err(format!(
            "{:?}. Expected a path with distance {:?}, but no path was found.",
            test.query(),
            expected_distance
        )),

        (Some(expected_distance), Some(actual_distance)) => {
            if expected_distance == actual_distance {
                Ok(())
            } else {
                Err(format!(
                    "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
                    test.query(),
                    expected_distance,
                    actual_distance
                ))
            }
        }
    }
}

fn validate_path(
    graph: &FlattenedNested<WeightedEdge>,
    test: &PathDistance,
    path: &Option<Path>,
) -> Result<(), String> {
    let actual_distance = path.as_ref().map(|path| path.distance().clone());
    validate_distance(test, &actual_distance)?;

    if let (Some(path), Some(expected_distance)) = (path.as_ref(), test.distance()) {
        validate_found_path(graph, path, test.query(), expected_distance)?;
    }

    Ok(())
}

fn validate_found_path(
    graph: &FlattenedNested<WeightedEdge>,
    path: &Path,
    query: &PathQuery,
    expected: Distance,
) -> Result<(), String> {
    let source = query.source();
    let target = query.target();

    if path.distance() != expected {
        return Err(format!(
            "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
            query,
            expected,
            path.distance(),
        ));
    }

    let vertices = path.vertices();

    if vertices.first() != Some(&source) {
        return Err(format!(
            "{:?}. Path starts at {:?}, expected {:?}.",
            query,
            vertices.first(),
            source,
        ));
    }

    if vertices.last() != Some(&target) {
        return Err(format!(
            "{:?}. Path ends at {:?}, expected {:?}.",
            query,
            vertices.last(),
            target,
        ));
    }

    let Some(sum) = sum_leaf_path(vertices, graph) else {
        return Err(format!(
            "{:?}. Path contains a non-leaf or missing edge: {:?}.",
            query, vertices,
        ));
    };

    if sum != expected {
        return Err(format!(
            "{:?}. Expanded path sum mismatch: expected {:?}, but got {:?}",
            query, expected, sum
        ));
    }

    Ok(())
}

fn sum_leaf_path(path: &[VertexId], graph: &FlattenedNested<WeightedEdge>) -> Option<Distance> {
    let mut sum = Distance::ZERO;

    for window in path.windows(2) {
        sum = sum + leaf_edge_weight(graph, window[0], window[1])?;
    }

    Some(sum)
}

fn leaf_edge_weight(
    graph: &FlattenedNested<WeightedEdge>,
    tail: VertexId,
    head: VertexId,
) -> Option<Distance> {
    find_leaf_edge(graph, tail, head).map(|edge| edge.weight)
}

fn find_leaf_edge(
    graph: &FlattenedNested<WeightedEdge>,
    tail: VertexId,
    head: VertexId,
) -> Option<&WeightedEdge> {
    if tail.as_usize() >= graph.num_nested() {
        return None;
    }

    graph
        .nested(tail.as_usize())
        .iter()
        .filter(|edge| edge.head == head)
        .min_by_key(|edge| edge.weight)
}
