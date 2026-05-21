use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use ch::{
    classical_search::DijkstraPathfinder,
    data_structures::VecSearchState,
    graph::{CsrGraph, WeightedEdge},
    path::PathDistance,
    types::VertexId,
    validation::{validate_distances, validate_paths},
};
use ordered_float::OrderedFloat;

pub fn edges_from_dimacs<R, VertexParser, WeightParser, EdgeCreator, VertexType, WeightType, Edge>(
    reader: R,
    vertex_parser: VertexParser,
    weight_parser: WeightParser,
    edge_creator: EdgeCreator,
) -> Option<Vec<Edge>>
where
    R: Read,
    VertexParser: Fn(&str) -> Option<VertexType>,
    WeightParser: Fn(&str) -> Option<WeightType>,
    EdgeCreator: Fn(VertexType, VertexType, WeightType) -> Edge,
{
    let mut lines = BufReader::new(reader).lines().filter_map(|line| {
        let line = line.ok()?;
        let line = line.trim().to_string();

        (!line.is_empty() && !line.starts_with('c')).then_some(line)
    });

    let problem = lines.next()?;
    let parts: Vec<&str> = problem.split_whitespace().collect();

    (parts.get(0)? == &"p" && parts.get(1)? == &"sp").then_some(())?;

    let _node_count: usize = parts.get(2)?.parse().ok()?;
    let edge_count: usize = parts.get(3)?.parse().ok()?;

    let mut edges = Vec::with_capacity(edge_count);

    for line in lines.take(edge_count) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        (parts.get(0)? == &"a").then_some(())?;

        let tail = vertex_parser(parts.get(1)?)?;
        let head = vertex_parser(parts.get(2)?)?;
        let weight = weight_parser(parts.get(3)?)?;

        edges.push(edge_creator(tail, head, weight));
    }

    (edges.len() == edge_count).then_some(edges)
}

#[test]
fn dummy_test() {
    type DistanceType = OrderedFloat<f64>;
    let epsilon = OrderedFloat::<f64>(1e-6);

    let edges = edges_from_dimacs(
        BufReader::new(File::open("tests/fixtures/karlsruhe.gr").unwrap()),
        |vertex_parser| vertex_parser.parse::<VertexId>().ok(),
        |weight_parser| weight_parser.parse::<DistanceType>().ok(),
        |tail, head, weight| WeightedEdge { tail, head, weight },
    )
    .unwrap();

    let tests_input = File::open("tests/fixtures/karlsruhe_tests.json").unwrap();
    let tests: Vec<PathDistance<DistanceType>> =
        serde_json::from_reader(BufReader::new(tests_input)).unwrap();

    let graph = CsrGraph::from_flat(edges.clone());
    let mut pathfinder = DijkstraPathfinder::<_, VecSearchState<_>>::new(&graph);

    validate_distances(&tests, &mut pathfinder, epsilon).unwrap();

    validate_paths(&tests, &edges, &mut pathfinder, epsilon).unwrap();
}
