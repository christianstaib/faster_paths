use crate::{
    graph::{Edge, EdgeLike, GraphLike},
    path::{Path, PathDistance, PathQuery},
    pathfinder::ShortestPathFinder,
    types::{Distance, VertexId},
};
use num_traits::Zero;
use rand::seq::index::sample;
use std::{
    iter,
    time::{Duration, Instant},
};

/// Validates that `actual` matches the distance provided by `test`.
pub fn validate_distance<D: Distance>(
    test: &PathDistance<D>,
    actual: &Option<D>,
) -> Result<(), String> {
    if &test.distance() == actual {
        return Ok(());
    }

    Err(format!(
        "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
        test.query(),
        test.distance(),
        actual
    ))
}

pub fn validate_path<G: GraphLike>(
    graph: &G,
    test: &PathDistance<<G::Edge as EdgeLike>::Distance>,
    path: &Option<Path<<G::Edge as EdgeLike>::Distance>>,
) -> Result<(), String> {
    match (path, test.distance()) {
        (None, None) => Ok(()),

        (None, Some(expected_distance)) => Err(format!(
            "{:?}. Expected a path with distance {:?}, but no path was found.",
            test.query(),
            expected_distance
        )),

        (Some(found_path), None) => Err(format!(
            "{:?}. Expected no path, but found one with distance {:?}.",
            test.query(),
            found_path.distance
        )),

        (Some(found_path), Some(expected_distance)) => {
            validate_found_path(graph, test, found_path, expected_distance)
        }
    }
}

/// Validates all tests against `pathfinder`.
///
/// If `graph` is present, complete returned paths are checked against the graph. Otherwise only
/// distances are checked. On success, returns the average pathfinder runtime per test.
pub fn validate<D, G, P>(
    tests: &[PathDistance<D>],
    graph: Option<&G>,
    pathfinder: &mut P,
) -> Result<Duration, Vec<String>>
where
    D: Distance,
    G: GraphLike,
    G::Edge: EdgeLike<Distance = D>,
    P: ShortestPathFinder<Distance = D>,
{
    let mut total_runtime = Duration::ZERO;
    let mut failures = Vec::new();

    match graph {
        Some(graph) => {
            for test in tests {
                let start = Instant::now();
                let path = pathfinder.path(test.query());
                total_runtime += start.elapsed();

                if let Err(message) = validate_path(graph, test, &path) {
                    failures.push(message);
                }
            }
        }

        None => {
            for test in tests {
                let start = Instant::now();
                let distance = pathfinder.distance(test.query());
                total_runtime += start.elapsed();

                if let Err(message) = validate_distance(test, &distance) {
                    failures.push(message);
                }
            }
        }
    }

    if failures.is_empty() {
        Ok(average_runtime(total_runtime, tests.len()))
    } else {
        Err(failures)
    }
}

fn average_runtime(total_runtime: Duration, num_tests: usize) -> Duration {
    if num_tests == 0 {
        Duration::ZERO
    } else {
        total_runtime / num_tests as u32
    }
}

/// Sum up the edge weights of `path` in `graph`. If an edge is missing, return it as `Err`.
fn sum_edge_weights<G: GraphLike>(
    graph: &G,
    path: &[VertexId],
) -> Result<<G::Edge as EdgeLike>::Distance, Edge> {
    path.windows(2).try_fold(
        <G::Edge as EdgeLike>::Distance::zero(),
        |summed_distance, potential_edge| {
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
        },
    )
}

fn validate_found_path<G: GraphLike>(
    graph: &G,
    test: &PathDistance<<G::Edge as EdgeLike>::Distance>,
    path: &Path<<G::Edge as EdgeLike>::Distance>,
    expected_distance: <G::Edge as EdgeLike>::Distance,
) -> Result<(), String> {
    if path.distance != expected_distance {
        return Err(format!(
            "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
            test.query(),
            expected_distance,
            path.distance,
        ));
    }

    let vertices = &path.vertices;
    let query = test.query();

    if vertices.first() != Some(&query.source) {
        return Err(format!(
            "{:?}. Path starts at {:?}, expected {:?}.",
            query,
            vertices.first(),
            query.source,
        ));
    }

    if vertices.last() != Some(&query.target) {
        return Err(format!(
            "{:?}. Path ends at {:?}, expected {:?}.",
            query,
            vertices.last(),
            query.target,
        ));
    }

    let actual_sum = sum_edge_weights(graph, vertices).map_err(|missing_edge| {
        format!(
            "{:?}. Path contains missing edge: {:?} -> {:?}.",
            query, missing_edge.tail, missing_edge.head,
        )
    })?;

    if actual_sum != expected_distance {
        return Err(format!(
            "{:?}. Path edge weight sum mismatch: expected {:?}, but got {:?}.",
            query, expected_distance, actual_sum,
        ));
    }

    Ok(())
}

pub fn generate_queries(num_vertices: usize, num_tests: usize) -> Vec<PathQuery> {
    let mut rng = rand::rng();
    iter::repeat_with(|| {
        let [source, target] = sample(&mut rng, num_vertices, 2)
            .into_vec()
            .try_into()
            .unwrap();

        PathQuery {
            source: VertexId::new(source as u32),
            target: VertexId::new(target as u32),
        }
    })
    .take(num_tests)
    .collect()
}
