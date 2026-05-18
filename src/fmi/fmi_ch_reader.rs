use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use serde::de::DeserializeOwned;

use crate::{
    contraction_hierachy::{ContractionEdge, ContractionHierarchy},
    graph::FastGraph,
    types::{Distance, VertexId},
};

fn parse_ch_edge<D: Distance + FromStr>(line: &str) -> Option<ContractionEdge<D>> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() != 4 {
        return None;
    }

    let tail = VertexId::new(parts[0].parse().ok()?);
    let head = VertexId::new(parts[1].parse().ok()?);
    let weight = parts[2].parse().ok()?;
    let skipped = parse_skipped(parts[3]);

    Some(ContractionEdge {
        tail,
        head,
        weight,
        skipped,
    })
}

fn parse_skipped(raw: &str) -> Option<VertexId> {
    match raw {
        "None" | "-1" => None,
        vertex => vertex.parse().ok().map(VertexId::new),
    }
}

fn read_ch_edges<D: Distance + FromStr>(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
) -> Option<Vec<Vec<ContractionEdge<D>>>> {
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

pub(super) fn ch_from_text_reader<D: Distance + FromStr, R: std::io::Read>(
    reader: R,
) -> Option<ContractionHierarchy<D>> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_up_edges = lines.next()?.parse().ok()?;
    let num_down_edges = lines.next()?.parse().ok()?;

    Some(ContractionHierarchy::new(
        FastGraph::new(&read_ch_edges(&mut lines, num_up_edges)?),
        FastGraph::new(&read_ch_edges(&mut lines, num_down_edges)?),
    ))
}

pub fn read_fmi_ch<D: Distance + DeserializeOwned>(file: &Path) -> Option<ContractionHierarchy<D>> {
    let reader = BufReader::new(File::open(file).ok()?);
    let mut buffer = [];
    postcard::from_io((reader, &mut buffer))
        .ok()
        .map(|(ch, _)| ch)
}

/// Reads CH files produced by `/home/christianstaib/Downloads/ch_constructor`.
///
/// The constructor writes:
/// - number of upward edges
/// - number of downward edges
/// - upward edges as `tail head weight skipped`
/// - downward edges as `tail head weight skipped`
///
/// Missing skipped vertices are encoded as `-1`.
pub fn read_ch_constructor_ch<D: Distance + FromStr>(
    file: &Path,
) -> Option<ContractionHierarchy<D>> {
    ch_from_text_reader(BufReader::new(File::open(file).ok()?))
}
