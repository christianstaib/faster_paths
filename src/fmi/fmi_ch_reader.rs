use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    str::FromStr,
};

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

pub(super) fn ch_from_reader<D: Distance + FromStr, R: Read>(
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

pub fn read_fmi_ch<D: Distance + FromStr>(file: &Path) -> Option<ContractionHierarchy<D>> {
    let reader = BufReader::new(File::open(file).ok()?);
    ch_from_reader(reader)
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
    read_fmi_ch(file)
}

#[cfg(test)]
mod tests {
    use crate::{fmi::fmi_ch_reader::ch_from_reader, graph::GraphLike, types::VertexId};

    #[test]
    fn reads_ch_constructor_output() {
        let input = "\
2
1
0 2 7 -1
1 3 11 2
3 1 11 2
";

        let ch = ch_from_reader::<u32, _>(input.as_bytes()).unwrap();

        assert_eq!(ch.up_graph().num_edges(), 2);
        assert_eq!(ch.down_graph().num_edges(), 1);

        let up_from_zero = ch.up_graph().out_edges(VertexId::new(0));
        assert_eq!(up_from_zero[0].head, VertexId::new(2));
        assert_eq!(up_from_zero[0].weight, 7);
        assert_eq!(up_from_zero[0].skipped, None);

        let up_from_one = ch.up_graph().out_edges(VertexId::new(1));
        assert_eq!(up_from_one[0].head, VertexId::new(3));
        assert_eq!(up_from_one[0].weight, 11);
        assert_eq!(up_from_one[0].skipped, Some(VertexId::new(2)));

        let down_from_three = ch.down_graph().out_edges(VertexId::new(3));
        assert_eq!(down_from_three[0].head, VertexId::new(1));
        assert_eq!(down_from_three[0].skipped, Some(VertexId::new(2)));
    }
}
