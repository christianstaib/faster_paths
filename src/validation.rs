use crate::{
    graph::EdgeLike,
    path::{Path, Query},
    pathfinder::ShortestPathFinder,
    types::{Distance, Vertex, abs_diff_eq},
};
use num_traits::Zero;
use rand::seq::index::sample;
use serde::{Deserialize, Serialize};
use std::{
    iter,
    time::{Duration, Instant},
};

/// Test case for validating shortest path queries.
///
/// The expected distance is optional because the target might be unreachable
/// from the source.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PathTestCase<D: Distance> {
    pub query: Query,
    pub distance: Option<D>,
}

/// Validates paths returned by `pathfinder` against `tests`.
///
/// A test passes when:
/// - unreachable queries return no path,
/// - reachable queries return a path,
/// - the path distance matches the expected distance within `epsilon`,
/// - the path starts at the query source,
/// - the path ends at the query target,
/// - every consecutive vertex pair in the path exists in `edges`, and
/// - the summed edge weights match the expected distance within `epsilon`.
///
/// Returns the average query runtime when all paths are valid, or the collected
/// validation failure messages otherwise.
pub fn validate_paths<E, P>(
    tests: &[PathTestCase<P::Distance>],
    edges: &[E],
    pathfinder: &mut P,
    epsilon: P::Distance,
) -> Result<Duration, Vec<String>>
where
    E: EdgeLike<Weight = P::Distance>,
    P: ShortestPathFinder,
{
    let mut total_runtime = Duration::ZERO;
    let mut failures = Vec::new();

    for test in tests {
        let start = Instant::now();
        let path = pathfinder.path(&test.query);
        total_runtime += start.elapsed();

        if let Err(message) = validate_path(edges, test, &path, epsilon) {
            failures.push(message);
        }
    }

    if failures.is_empty() {
        Ok(total_runtime / tests.len() as u32)
    } else {
        Err(failures)
    }
}

/// Validates distances returned by `pathfinder` against `tests`.
///
/// Returns the average query runtime when all distances match within `epsilon`,
/// or the collected validation failure messages otherwise.
pub fn validate_distances<P>(
    tests: &[PathTestCase<P::Distance>],
    pathfinder: &mut P,
    epsilon: P::Distance,
) -> Result<Duration, Vec<String>>
where
    P: ShortestPathFinder,
{
    let mut total_runtime = Duration::ZERO;
    let mut failures = Vec::new();

    for test in tests {
        let start = Instant::now();
        let distance = pathfinder.distance(&test.query);
        total_runtime += start.elapsed();

        if let Err(message) = validate_distance(test, &distance, epsilon) {
            failures.push(message);
        }
    }

    if failures.is_empty() {
        Ok(total_runtime / tests.len() as u32)
    } else {
        Err(failures)
    }
}

fn validate_distance<D>(
    test: &PathTestCase<D>,
    actual: &Option<D>,
    epsilon: D,
) -> Result<(), String>
where
    D: Distance,
{
    match (test.distance, *actual) {
        (None, None) => Ok(()),
        (Some(expected), Some(actual)) if abs_diff_eq(expected, actual, epsilon) => Ok(()),
        (expected, actual) => Err(format!(
            "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
            test.query, expected, actual
        )),
    }
}

fn validate_path<E>(
    edges: &[E],
    test: &PathTestCase<E::Weight>,
    path: &Option<Path<E::Weight>>,
    epsilon: E::Weight,
) -> Result<(), String>
where
    E: EdgeLike,
{
    let query = &test.query;

    match (path, test.distance) {
        (None, None) => Ok(()),

        (None, Some(expected_distance)) => Err(format!(
            "{:?}. Expected a path with distance {:?}, but no path was found.",
            query, expected_distance
        )),

        (Some(found_path), None) => Err(format!(
            "{:?}. Expected no path, but found one with distance {:?}.",
            query, found_path.distance
        )),

        (Some(found_path), Some(expected_distance)) => {
            validate_found_path(edges, query, expected_distance, found_path, epsilon)
        }
    }
}

fn validate_found_path<E>(
    edges: &[E],
    query: &Query,
    expected_distance: E::Weight,
    path: &Path<E::Weight>,
    epsilon: E::Weight,
) -> Result<(), String>
where
    E: EdgeLike,
{
    if !abs_diff_eq(path.distance, expected_distance, epsilon) {
        return Err(format!(
            "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
            query, expected_distance, path.distance,
        ));
    }

    let vertices = &path.vertices;

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

    let actual_sum = path_weight(edges, vertices).map_err(|(tail, head)| {
        format!(
            "{:?}. Path contains missing edge: {:?} -> {:?}.",
            query, tail, head,
        )
    })?;

    if !abs_diff_eq(actual_sum, expected_distance, epsilon) {
        return Err(format!(
            "{:?}. Path edge weight sum mismatch: expected {:?}, but got {:?}.",
            query, expected_distance, actual_sum,
        ));
    }

    Ok(())
}

fn path_weight<E: EdgeLike>(
    edges: &[E],
    vertices: &[Vertex],
) -> Result<E::Weight, (Vertex, Vertex)> {
    let mut total = E::Weight::zero();

    for pair in vertices.windows(2) {
        let tail = pair[0];
        let head = pair[1];

        let Ok(index) =
            edges.binary_search_by_key(&(tail, head), |edge| (edge.tail(), edge.head()))
        else {
            return Err((tail, head));
        };

        total += edges[index].weight();
    }

    Ok(total)
}

pub fn generate_random_queries(num_vertices: usize, num_queries: usize) -> Vec<Query> {
    let mut rng = rand::rng();
    iter::repeat_with(|| {
        let [source, target] = sample(&mut rng, num_vertices, 2)
            .into_vec()
            .try_into()
            .unwrap();

        Query {
            source: source as u32,
            target: target as u32,
        }
    })
    .take(num_queries)
    .collect()
}
