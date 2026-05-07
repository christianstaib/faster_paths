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
