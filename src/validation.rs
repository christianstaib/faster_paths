use crate::{
    graph::{Edge, EdgeLike, GraphLike, WeightedEdge},
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

pub fn validate_path<G>(
    graph: &G,
    test: &PathDistance<<G::Edge as EdgeLike>::Weight>,
    path: &Option<Path<<G::Edge as EdgeLike>::Weight>>,
) -> Result<(), String>
where
    G: GraphLike,
{
    let edges: Vec<_> = graph
        .all_edges()
        .map(|edge| WeightedEdge {
            tail: edge.tail(),
            head: edge.head(),
            weight: edge.weight(),
        })
        .collect();

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
            validate_found_path(&edges, test.query(), &expected_distance, found_path)
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
    G::Edge: EdgeLike<Weight = D>,
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

    Ok(total_runtime / tests.len() as u32)
}

/// Sum up the edge weights of `path` in `edges`. If an edge is missing, return it as `Err`.
fn sum_edge_weights<E: EdgeLike>(edges: &Vec<E>, path: &[VertexId]) -> Result<E::Weight, Edge> {
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
    edges: &Vec<E>,
    query: &PathQuery,
    expected_distance: &D,
    path: &Path<D>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    validate_reported_distance(edges, query, expected_distance, path)?;

    validate_source_vertex(edges, query, expected_distance, path)?;

    validate_target_vertex(edges, query, expected_distance, path)?;

    validate_edge_weight_sum(edges, query, expected_distance, path)?;

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

fn validate_reported_distance<E, D>(
    _edges: &Vec<E>,
    query: &PathQuery,
    expected_distance: &D,
    path: &Path<D>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    if &path.distance == expected_distance {
        return Ok(());
    }

    Err(format!(
        "{:?}. Distance mismatch: expected {:?}, but got {:?}.",
        query, expected_distance, path.distance,
    ))
}

fn validate_source_vertex<E, D>(
    _edges: &Vec<E>,
    query: &PathQuery,
    _expected_distance: &D,
    path: &Path<D>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    if path.vertices.first() == Some(&query.source) {
        return Ok(());
    }

    Err(format!(
        "{:?}. Path starts at {:?}, expected {:?}.",
        query,
        path.vertices.first(),
        query.source,
    ))
}

fn validate_target_vertex<E, D>(
    _edges: &Vec<E>,
    query: &PathQuery,
    _expected_distance: &D,
    path: &Path<D>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    if path.vertices.last() == Some(&query.target) {
        return Ok(());
    }

    Err(format!(
        "{:?}. Path ends at {:?}, expected {:?}.",
        query,
        path.vertices.last(),
        query.target,
    ))
}

fn validate_edge_weight_sum<E, D>(
    edges: &Vec<E>,
    query: &PathQuery,
    expected_distance: &D,
    path: &Path<D>,
) -> Result<(), String>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    let actual_sum = sum_edge_weights(edges, &path.vertices).map_err(|missing_edge| {
        format!(
            "{:?}. Path contains missing edge: {:?} -> {:?}.",
            query, missing_edge.tail, missing_edge.head,
        )
    })?;

    if &actual_sum == expected_distance {
        return Ok(());
    }

    Err(format!(
        "{:?}. Path edge weight sum mismatch: expected {:?}, but got {:?}.",
        query, expected_distance, actual_sum,
    ))
}
