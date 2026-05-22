mod common;

use ch::{
    contraction_hierachy::contract_graph_sequential,
    hub_labeling::{HubLabeling, HubLabelingPathfinder},
    validation::{validate_distances, validate_paths},
};
use ordered_float::OrderedFloat;

#[test]
fn karlsruhe_fixture_matches_hub_labeling() {
    type DistanceType = OrderedFloat<f64>;
    let epsilon = OrderedFloat::<f64>(1e-6);

    let edges = common::karlsruhe_edges::<DistanceType>();
    let tests = common::karlsruhe_tests::<DistanceType>();

    let contraction_hierarchy = contract_graph_sequential(&edges);
    let hub_labeling =
        HubLabeling::try_from_contraction_hierarchy(&contraction_hierarchy, epsilon).unwrap();
    let mut pathfinder = HubLabelingPathfinder {
        contraction_hierarchy: &contraction_hierarchy,
        hub_labeling: &hub_labeling,
    };

    validate_distances(&tests, &mut pathfinder, epsilon).unwrap();

    validate_paths(&tests, &edges, &mut pathfinder, epsilon).unwrap();
}
