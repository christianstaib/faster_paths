use std::io::{self, Write};

use crate::{ch::ContractionHierarchy, graph::FastGraph, graph::GraphLike, types::VertexId};

pub fn write_fmi_ch<W: Write>(mut out: W, ch: &ContractionHierarchy) -> io::Result<()> {
    writeln!(out, "{}", ch.up_graph().num_edges())?;
    writeln!(out, "{}", ch.down_graph().num_edges())?;
    write_edges(&mut out, ch.up_graph())?;
    write_edges(&mut out, ch.down_graph())
}

fn write_edges<W: Write>(
    out: &mut W,
    graph: &FastGraph<crate::ch::ContractionEdge>,
) -> io::Result<()> {
    for tail in 0..graph.num_vertices() {
        for edge in graph.out_edges(VertexId::new(tail as u32)) {
            writeln!(
                out,
                "{} {} {} {}",
                edge.tail.as_usize(),
                edge.head.as_usize(),
                edge.weight.as_u32(),
                skipped(edge.skipped),
            )?;
        }
    }

    Ok(())
}

fn skipped(skipped: Option<VertexId>) -> String {
    skipped.map_or_else(|| "-1".to_owned(), |v| v.as_usize().to_string())
}
