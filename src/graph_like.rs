use crate::{edge_like::EdgeLike, types::VertexId};

pub trait GraphLike {
    type Edge: EdgeLike;

    fn out_edges(&self, tail: VertexId) -> &[Self::Edge];
    fn num_vertices(&self) -> usize;
    fn num_edges(&self) -> usize;
}
