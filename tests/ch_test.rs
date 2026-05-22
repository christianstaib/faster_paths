mod common;

use ch::{
    contraction_hierarchy::{ContractionHierarchyPathfinder, contract_graph_sequential},
    validation::{validate_distances, validate_paths},
};
use ordered_float::OrderedFloat;

#[test]
fn karlsruhe_fixture_matches_contraction_hierarchy() {
    type DistanceType = OrderedFloat<f64>;
    let epsilon = OrderedFloat::<f64>(1e-6);

    let edges = common::karlsruhe_edges::<DistanceType>();
    let tests = common::karlsruhe_tests::<DistanceType>();

    let contraction_hierarchy = contract_graph_sequential(&edges);
    let mut pathfinder = ContractionHierarchyPathfinder::new(&contraction_hierarchy);

    validate_distances(&tests, &mut pathfinder, epsilon).unwrap();

    validate_paths(&tests, &edges, &mut pathfinder, epsilon).unwrap();
}
