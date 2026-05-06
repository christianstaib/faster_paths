use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::{
    flattened_nested::FlattenedNested,
    graph::Edge,
    types::{Distance, VertexId},
};

fn parse_edge(line: &str) -> Option<Edge> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() < 3 {
        return None;
    }

    let tail = VertexId::new(parts[0].parse().ok()?);
    let head = VertexId::new(parts[1].parse().ok()?);
    let weight = Distance::new(parts[2].parse().ok()?);

    Some(Edge::new(tail, head, weight))
}

fn read_edges(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
    num_vertices: usize,
) -> Option<Vec<Vec<Edge>>> {
    let mut graph = Vec::new();
    graph.resize_with(num_vertices, Vec::new);

    for _ in 0..count {
        let edge = parse_edge(&next_data_line(lines)?)?;
        let tail = edge.tail.as_usize();

        if graph.len() <= tail {
            graph.resize_with(tail + 1, Vec::new);
        }

        graph[tail].push(edge);
    }

    Some(graph)
}

fn next_data_line(lines: &mut impl Iterator<Item = String>) -> Option<String> {
    lines.find(|line| {
        let line = line.trim();
        !line.is_empty() && !line.starts_with('#')
    })
}

fn graph_from_reader<R: Read>(reader: R) -> Option<FlattenedNested<Edge>> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_vertices: usize = next_data_line(&mut lines)?.trim().parse().ok()?;
    let num_edges = next_data_line(&mut lines)?.trim().parse().ok()?;

    for _ in 0..num_vertices {
        next_data_line(&mut lines)?;
    }

    Some(FlattenedNested::new(&read_edges(
        &mut lines,
        num_edges,
        num_vertices,
    )?))
}

pub fn read_fmi_graph(file: &std::path::Path) -> Option<FlattenedNested<Edge>> {
    let reader = BufReader::new(File::open(file).unwrap());
    graph_from_reader(reader)
}
