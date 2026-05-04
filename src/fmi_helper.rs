use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::{
    ch::contraction_hierarchy::ContractionHierarchy,
    flattened_nested::FlattenedNested,
    path::{PathDistance, PathQuery},
    types::{Distance, VertexId},
};

fn parse_ch_edge(line: &str) -> Option<crate::ch::edge::Edge> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() != 4 {
        return None;
    }

    let tail = VertexId::new(parts[0].parse().ok()?);
    let head = VertexId::new(parts[1].parse().ok()?);
    let weight = Distance::new(parts[2].parse().ok()?);
    let skiped = parts[3].parse().ok().map(|x| VertexId::new(x));

    Some(crate::ch::edge::Edge::new(tail, head, weight, skiped))
}

fn parse_edge(line: &str) -> Option<crate::edge::Edge> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() < 3 {
        return None;
    }

    let tail = VertexId::new(parts[0].parse().ok()?);
    let head = VertexId::new(parts[1].parse().ok()?);
    let weight = Distance::new(parts[2].parse().ok()?);

    Some(crate::edge::Edge::new(tail, head, weight))
}

fn read_edges(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
) -> Option<Vec<Vec<crate::edge::Edge>>> {
    let mut graph = Vec::new();

    for _ in 0..count {
        let edge = parse_edge(&lines.next()?)?;
        let tail = edge.tail().as_usize();

        if graph.len() <= tail {
            graph.resize_with(tail + 1, Vec::new);
        }

        graph[tail].push(edge);
    }

    Some(graph)
}

fn read_ch_edges(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
) -> Option<Vec<Vec<crate::ch::edge::Edge>>> {
    let mut graph = Vec::new();

    for _ in 0..count {
        let edge = parse_ch_edge(&lines.next()?)?;
        let tail = edge.tail().as_usize();

        if graph.len() <= tail {
            graph.resize_with(tail + 1, Vec::new);
        }

        graph[tail].push(edge);
    }

    Some(graph)
}

fn graph_from_reader<R: Read>(reader: R) -> Option<FlattenedNested<crate::edge::Edge>> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);
    while lines.by_ref().next().unwrap().starts_with('#') {}

    let _num_verties: usize = lines.next()?.parse().ok()?;
    let num_edges = lines.next()?.parse().ok()?;

    lines.by_ref().take(_num_verties).count();

    Some(FlattenedNested::new(read_edges(&mut lines, num_edges)?))
}

fn ch_from_reader<R: Read>(reader: R) -> Option<ContractionHierarchy> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_up_edges = lines.next()?.parse().ok()?;
    let num_down_edges = lines.next()?.parse().ok()?;

    Some(ContractionHierarchy::new(
        FlattenedNested::new(read_ch_edges(&mut lines, num_up_edges)?),
        FlattenedNested::new(read_ch_edges(&mut lines, num_down_edges)?),
    ))
}

pub fn read_fmi_ch(file: &std::path::Path) -> Option<ContractionHierarchy> {
    let reader = BufReader::new(File::open(file).unwrap());
    ch_from_reader(reader)
}

pub fn read_fmi_graph(file: &std::path::Path) -> Option<FlattenedNested<crate::edge::Edge>> {
    let reader = BufReader::new(File::open(file).unwrap());
    graph_from_reader(reader)
}

pub fn read_tests(file: &std::path::Path) -> Option<Vec<PathDistance>> {
    let mut tests = Vec::new();
    let reader_test = BufReader::new(File::open(&file).unwrap());
    let mut test_lines = reader_test.lines().flatten();
    test_lines.next();
    while let Some(line) = test_lines.next() {
        let mut parts = line.split_whitespace();

        let source = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
        let target = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
        let query = PathQuery::new(source, target);

        let distance: Option<Distance> =
            parts.next().unwrap().parse().ok().map(|x| Distance::new(x));

        let validation = PathDistance::new(query, distance);

        tests.push(validation);
    }

    Some(tests)
}
