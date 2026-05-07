use crate::{
    graph::{Edge, EdgeLike, GraphLike},
    path::{Path, PathDistance},
    types::{Distance, VertexId},
};
use num_traits::Zero;

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
