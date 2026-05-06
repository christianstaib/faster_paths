use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

use crate::{
    ch::{ContractionEdge, ContractionHierarchy},
    graph::FastGraph,
    types::{Distance, VertexId},
};

fn parse_ch_edge(line: &str) -> Option<ContractionEdge> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() != 4 {
        return None;
    }

    let tail = VertexId::new(parts[0].parse().ok()?);
    let head = VertexId::new(parts[1].parse().ok()?);
    let weight = Distance::new(parts[2].parse().ok()?);
    let skiped = parts[3].parse().ok().map(|x| VertexId::new(x));

    Some(ContractionEdge::new(tail, head, weight, skiped))
}

fn read_ch_edges(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
) -> Option<Vec<Vec<ContractionEdge>>> {
    let mut graph = Vec::new();

    for _ in 0..count {
        let edge = parse_ch_edge(&lines.next()?)?;

        let tail = edge.tail.as_usize();
        if graph.len() <= tail {
            graph.resize_with(tail + 1, Vec::new);
        }

        graph[tail].push(edge);
    }

    Some(graph)
}

fn ch_from_reader<R: Read>(reader: R) -> Option<ContractionHierarchy> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_up_edges = lines.next()?.parse().ok()?;
    let num_down_edges = lines.next()?.parse().ok()?;

    Some(ContractionHierarchy::new(
        FastGraph::new(&read_ch_edges(&mut lines, num_up_edges)?),
        FastGraph::new(&read_ch_edges(&mut lines, num_down_edges)?),
    ))
}

pub fn read_fmi_ch(file: &std::path::Path) -> Option<ContractionHierarchy> {
    let reader = BufReader::new(File::open(file).unwrap());
    ch_from_reader(reader)
}
