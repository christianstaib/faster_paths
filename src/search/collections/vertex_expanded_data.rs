use std::usize;

use fixedbitset::FixedBitSet;

use crate::graphs::{Graph, Vertex};

pub trait VertexExpandedData {
    fn expand(&mut self, vertex: Vertex) -> bool;

    fn clear(&mut self);
}

pub struct VertexExpandedDataVec {
    expanded: Vec<bool>,
}

impl VertexExpandedDataVec {
    pub fn new(graph: &dyn Graph) -> Self {
        VertexExpandedDataVec {
            expanded: vec![false; graph.number_of_vertices() as usize],
        }
    }
}

impl VertexExpandedData for VertexExpandedDataVec {
    fn expand(&mut self, vertex: Vertex) -> bool {
        let is_expanded = self.expanded[vertex as usize];
        self.expanded[vertex as usize] = true;
        is_expanded
    }

    fn clear(&mut self) {
        self.expanded.fill(false);
    }
}

pub struct VertexExpandedDataBitSet {
    expanded: FixedBitSet,
}

impl VertexExpandedDataBitSet {
    pub fn new(graph: &dyn Graph) -> Self {
        VertexExpandedDataBitSet {
            expanded: FixedBitSet::with_capacity(graph.number_of_vertices() as usize),
        }
    }
}

impl VertexExpandedData for VertexExpandedDataBitSet {
    fn expand(&mut self, vertex: Vertex) -> bool {
        self.expanded.put(vertex as usize)
    }

    fn clear(&mut self) {
        self.expanded.clear()
    }
}
