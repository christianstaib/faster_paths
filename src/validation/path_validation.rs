use crate::{
    graph::{Edge, EdgeLike},
    path::{Path, PathDistance, PathQuery},
    types::{Distance, VertexId},
};
use num_traits::Zero;

pub type PathCheck<E, D> = fn(Vec<E>, &PathQuery, &D, &Path<D>) -> Result<(), String>;

pub type NamedPathCheck<E, D> = (&'static str, PathCheck<E, D>);

pub fn default_path_checks<E, D>() -> Vec<NamedPathCheck<E, D>>
where
    D: Distance,
    E: Clone + EdgeLike<Weight = D>,
{
    vec![
        ("distance", validate_reported_distance::<E, D>),
        ("source", validate_source_vertex::<E, D>),
        ("target", validate_target_vertex::<E, D>),
        ("edges", validate_edge_weight_sum::<E, D>),
    ]
}

pub fn validate_path<E, D>(
    edges: &[E],
    test: &PathDistance<D>,
    path: &Option<Path<D>>,
) -> Result<(), Vec<String>>
where
    D: Distance,
    E: Clone,
    E: EdgeLike<Weight = D>,
{
    validate_path_with_checks(edges, test, path, &default_path_checks())
}

pub fn validate_path_with_checks<E, D>(
    edges: &[E],
    test: &PathDistance<D>,
    path: &Option<Path<D>>,
    checks: &[NamedPathCheck<E, D>],
) -> Result<(), Vec<String>>
where
    D: Distance,
    E: Clone,
{
    match (path, test.distance()) {
        (None, None) => Ok(()),
        (None, Some(distance)) => Err(vec![format!(
            "{:?}. Expected a path with distance {:?}, but no path was found.",
            test.query(),
            distance
        )]),
        (Some(path), None) => Err(vec![format!(
            "{:?}. Expected no path, but found one with distance {:?}.",
            test.query(),
            path.distance
        )]),
        (Some(path), Some(distance)) => {
            run_path_checks(edges, test, path, distance, checks).map(|_| ())
        }
    }
}

fn run_path_checks<E, D>(
    edges: &[E],
    test: &PathDistance<D>,
    path: &Path<D>,
    expected_distance: D,
    checks: &[NamedPathCheck<E, D>],
) -> Result<(), Vec<String>>
where
    D: Distance,
    E: Clone,
{
    let failures: Vec<_> = checks
        .iter()
        .filter_map(|(name, check)| {
            check(edges.to_vec(), test.query(), &expected_distance, path)
                .err()
                .map(|message| format!("{name}: {message}"))
        })
        .collect();

    match failures.is_empty() {
        true => Ok(()),
        false => Err(failures),
    }
}
