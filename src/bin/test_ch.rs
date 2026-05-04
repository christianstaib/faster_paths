use ch::ch::contraction_hierarchy::ContractionHierarchy;
use ch::ch::edge::Edge;
use ch::ch::pathfinder::ContractionHierarchyPathfinder;
use ch::flattened_nested::FlattenedNested;
use ch::fmi_helper::{read_fmi_ch, read_tests};
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
    graph_in: PathBuf,

    /// Test queries in .txt format
    #[arg(short, long)]
    test_in: PathBuf,
}

fn main() {
    let args = Args::parse();

    let ch = read_fmi_ch(&args.graph_in).unwrap();
    let graph = leaf_graph(&ch);
    let tests = read_tests(&args.test_in).unwrap();
    let mut pathfinder = ContractionHierarchyPathfinder::new(&ch);

    let mut failures = 0;
    for test in &tests {
        let path = pathfinder.path(test.query());
        if let Err(message) = validate_path(&graph, test, &path) {
            failures += 1;
            eprintln!("{message}");
        }
    }

    if failures == 0 {
        println!("All {} paths correct.", tests.len());
    } else {
        println!("{failures} of {} paths failed.", tests.len());
        std::process::exit(1);
    }
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

fn leaf_graph(ch: &ContractionHierarchy) -> FlattenedNested<Edge> {
    let mut graph = vec![Vec::new(); ch.up_graph().num_nested().max(ch.down_graph().num_nested())];

    for tail in 0..ch.up_graph().num_nested() {
        for edge in ch.up_graph().nested(tail) {
            if edge.skipped().is_none() {
                push_leaf_edge(&mut graph, *edge);
            }
        }
    }

    for tail in 0..ch.down_graph().num_nested() {
        for edge in ch.down_graph().nested(tail) {
            if edge.skipped().is_none() {
                push_leaf_edge(
                    &mut graph,
                    Edge::new(edge.head(), edge.tail(), edge.weight(), None),
                );
            }
        }
    }

    FlattenedNested::new(graph)
}

fn push_leaf_edge(graph: &mut Vec<Vec<Edge>>, edge: Edge) {
    let tail = edge.tail().as_usize();
    if graph.len() <= tail {
        graph.resize_with(tail + 1, Vec::new);
    }

    graph[tail].push(edge);
}

fn validate_path(
    graph: &FlattenedNested<Edge>,
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
    graph: &FlattenedNested<Edge>,
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
            "{:?}. Expanded path sum mismatch: expected {:?}, but got {:?}; path {:?}.",
            query, expected, sum, vertices,
        ));
    }

    Ok(())
}

fn sum_leaf_path(path: &[VertexId], graph: &FlattenedNested<Edge>) -> Option<Distance> {
    let mut sum = Distance::ZERO;

    for window in path.windows(2) {
        sum = sum + leaf_edge_weight(graph, window[0], window[1])?;
    }

    Some(sum)
}

fn leaf_edge_weight(
    graph: &FlattenedNested<Edge>,
    tail: VertexId,
    head: VertexId,
) -> Option<Distance> {
    find_leaf_edge(graph, tail, head).map(Edge::weight)
}

fn find_leaf_edge(graph: &FlattenedNested<Edge>, tail: VertexId, head: VertexId) -> Option<&Edge> {
    if tail.as_usize() >= graph.num_nested() {
        return None;
    }

    graph
        .nested(tail.as_usize())
        .iter()
        .find(|edge| edge.head() == head && edge.skipped().is_none())
}
