use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use ch::{graph::WeightedEdge, path::PathDistance, types::Vertex};
use serde::de::DeserializeOwned;

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

    (parts.first()? == &"p" && parts.get(1)? == &"sp").then_some(())?;

    let _node_count: usize = parts.get(2)?.parse().ok()?;
    let edge_count: usize = parts.get(3)?.parse().ok()?;

    let mut edges = Vec::with_capacity(edge_count);

    for line in lines.take(edge_count) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        (parts.first()? == &"a").then_some(())?;

        let tail = vertex_parser(parts.get(1)?)?;
        let head = vertex_parser(parts.get(2)?)?;
        let weight = weight_parser(parts.get(3)?)?;

        edges.push(edge_creator(tail, head, weight));
    }

    (edges.len() == edge_count).then_some(edges)
}

pub fn karlsruhe_edges<D>() -> Vec<WeightedEdge<D>>
where
    D: FromStr,
{
    edges_from_dimacs(
        BufReader::new(File::open("tests/fixtures/karlsruhe.gr").unwrap()),
        |vertex_parser| vertex_parser.parse::<Vertex>().ok(),
        |weight_parser| weight_parser.parse::<D>().ok(),
        |tail, head, weight| WeightedEdge { tail, head, weight },
    )
    .unwrap()
}

pub fn karlsruhe_tests<D>() -> Vec<PathDistance<D>>
where
    D: ch::types::Distance + DeserializeOwned,
{
    let tests_input = File::open("tests/fixtures/karlsruhe_tests.json").unwrap();
    serde_json::from_reader(BufReader::new(tests_input)).unwrap()
}
