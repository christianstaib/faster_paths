use crate::graphs::{Graph, VertexId};

pub trait VertexExpandedData {
    /// Returns true the first the a vertex expanded, false afterwards
    fn expand(&mut self, vertex: VertexId) -> bool;

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
    fn expand(&mut self, vertex: VertexId) -> bool {
        let expanded = self.expanded[vertex as usize];
        self.expanded[vertex as usize] = true;
        expanded
    }

    fn clear(&mut self) {
        self.expanded.fill(false);
    }
}
