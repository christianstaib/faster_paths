use std::{
    fmt::Display,
    io::{self, Write},
};

use crate::{
    contraction_hierachy::{ContractionEdge, ContractionHierarchy},
    graph::{FastGraph, GraphLike},
    types::{Distance, VertexId},
};

pub fn write_fmi_ch<W: Write, D: Distance + Display>(
    mut out: W,
    ch: &ContractionHierarchy<D>,
) -> io::Result<()> {
    writeln!(out, "{}", ch.up_graph().num_edges())?;
    writeln!(out, "{}", ch.down_graph().num_edges())?;
    write_edges(&mut out, ch.up_graph())?;
    write_edges(&mut out, ch.down_graph())
}

fn write_edges<W: Write, D: Distance + Display>(
    out: &mut W,
    graph: &FastGraph<ContractionEdge<D>>,
) -> io::Result<()> {
    for edge in graph.edges() {
        writeln!(
            out,
            "{} {} {} {}",
            edge.tail.as_usize(),
            edge.head.as_usize(),
            edge.weight,
            skipped(edge.skipped),
        )?;
    }

    Ok(())
}

fn skipped(skipped: Option<VertexId>) -> String {
    skipped.map_or_else(|| "None".to_owned(), |v| v.as_usize().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VertexId;

    #[test]
    fn writes_none_for_unskipped_edge() {
        let up_edges = vec![vec![ContractionEdge::new(
            VertexId::new(0),
            VertexId::new(1),
            5_u32,
            None,
        )]];
        let down_edges: Vec<Vec<ContractionEdge<u32>>> = vec![Vec::new()];
        let ch = ContractionHierarchy::new(FastGraph::new(&up_edges), FastGraph::new(&down_edges));

        let mut out = Vec::new();
        write_fmi_ch(&mut out, &ch).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), "1\n0\n0 1 5 None\n");
    }
}
