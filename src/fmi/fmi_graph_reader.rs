use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use crate::{
    graph::{FastGraph, WeightedEdge},
    types::{Distance, VertexId},
};

fn parse_edge<D: Distance + FromStr>(line: &str) -> Option<WeightedEdge<D>> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() < 3 {
        return None;
    }

    let tail = VertexId::new(parts[0].parse().ok()?);
    let head = VertexId::new(parts[1].parse().ok()?);
    let weight = parts[2].parse().ok()?;

    Some(WeightedEdge { tail, head, weight })
}

fn read_edges<D: Distance + FromStr>(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
    num_vertices: usize,
) -> Option<Vec<Vec<WeightedEdge<D>>>> {
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

fn graph_from_reader<D: Distance + FromStr, R: Read>(
    reader: R,
) -> Option<FastGraph<WeightedEdge<D>>> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_vertices: usize = next_data_line(&mut lines)?.trim().parse().ok()?;
    let num_edges = next_data_line(&mut lines)?.trim().parse().ok()?;

    for _ in 0..num_vertices {
        next_data_line(&mut lines)?;
    }

    Some(FastGraph::new(&read_edges(
        &mut lines,
        num_edges,
        num_vertices,
    )?))
}

pub fn read_fmi_graph<D: Distance + FromStr>(
    file: &std::path::Path,
) -> Option<FastGraph<WeightedEdge<D>>> {
    let reader = BufReader::new(File::open(file).unwrap());
    graph_from_reader(reader)
}
