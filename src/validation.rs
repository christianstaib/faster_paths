use crate::{
    graph::{Edge, EdgeLike},
    path::{Path, PathDistance, PathQuery},
    pathfinder::ShortestPathFinder,
    types::{Distance, VertexId},
};
use num_traits::Zero;
use std::time::{Duration, Instant};

/// Validates all tests against `pathfinder`.
///
/// If `edges` is present, complete returned paths are checked against the edges. Otherwise only
/// distances are checked. On success, returns the average pathfinder runtime per test.
pub fn validate_paths<D, E, P>(
    tests: &[PathDistance<D>],
    edges: &[E],
    pathfinder: &mut P,
) -> Result<Duration, Vec<String>>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
    P: ShortestPathFinder<Distance = D>,
{
    let mut total_runtime = Duration::ZERO;
    let mut failures = Vec::new();

    for test in tests {
        let start = Instant::now();
        let path = pathfinder.path(test.query());
        total_runtime += start.elapsed();

        if let Err(message) = validate_path(edges, test, &path) {
            failures.push(message);
        }
    }

    if !failures.is_empty() {
        return Err(failures);
    }

    Ok(total_runtime / tests.len() as u32)
}

/// Validates all tests against `pathfinder`.
///
/// If `edges` is present, complete returned paths are checked against the edges. Otherwise only
/// distances are checked. On success, returns the average pathfinder runtime per test.
pub fn validate_distances<D, E, P>(
    tests: &[PathDistance<D>],
    pathfinder: &mut P,
) -> Result<Duration, Vec<String>>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
    P: ShortestPathFinder<Distance = D>,
{
    let mut total_runtime = Duration::ZERO;
    let mut failures = Vec::new();

    for test in tests {
        let start = Instant::now();
        let distance = pathfinder.distance(test.query());
        total_runtime += start.elapsed();

        if let Err(message) = validate_distance(test, &distance) {
            failures.push(message);
        }
    }

    if !failures.is_empty() {
        return Err(failures);
    }

    Ok(total_runtime / tests.len() as u32)
}

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

pub fn validate_path<E, D>(
    edges: &[E],
    test: &PathDistance<D>,
    path: &Option<Path<D>>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
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
            validate_found_path(edges, test.query(), &expected_distance, found_path)
        }
    }
}

/// Sum up the edge weights of `path` in `edges`. If an edge is missing, return it as `Err`.
fn sum_edge_weights<E: EdgeLike>(edges: &[E], path: &[VertexId]) -> Result<E::Weight, Edge> {
    path.windows(2)
        .try_fold(E::Weight::zero(), |summed_distance, potential_edge| {
            let tail = potential_edge[0];
            let head = potential_edge[1];

            let weight = edges
                .iter()
                .filter(|edge| edge.tail() == tail && edge.head() == head)
                .map(|edge| edge.weight())
                .min()
                .ok_or(Edge { tail, head })?;

            Ok(summed_distance + weight)
        })
}

fn validate_found_path<E: EdgeLike, D: Distance>(
    edges: &[E],
    query: &PathQuery,
    expected_distance: &D,
    path: &Path<D>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    if &path.distance != expected_distance {
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

    let actual_sum = sum_edge_weights(edges, vertices).map_err(|missing_edge| {
        format!(
            "{:?}. Path contains missing edge: {:?} -> {:?}.",
            query, missing_edge.tail, missing_edge.head,
        )
    })?;

    if &actual_sum != expected_distance {
        return Err(format!(
            "{:?}. Path edge weight sum mismatch: expected {:?}, but got {:?}.",
            query, expected_distance, actual_sum,
        ));
    }

    Ok(())
}
