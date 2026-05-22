mod common;

use faster_paths::{
    classical_search::DijkstraPathfinder,
    data_structures::VecSearchState,
    graph::CsrGraph,
    validation::{validate_distances, validate_paths},
};
use ordered_float::OrderedFloat;

#[test]
fn karlsruhe_fixture_matches_dijkstra() {
    type DistanceType = OrderedFloat<f64>;
    let epsilon = OrderedFloat::<f64>(1e-6);

    let edges = common::karlsruhe_edges::<DistanceType>();
    let tests = common::karlsruhe_tests::<DistanceType>();

    let graph = CsrGraph::from_flat(edges.clone());
    let mut pathfinder = DijkstraPathfinder::<_, VecSearchState<_>>::new(&graph);

    validate_distances(&tests, &mut pathfinder, epsilon).unwrap();

    validate_paths(&tests, &edges, &mut pathfinder, epsilon).unwrap();
}
