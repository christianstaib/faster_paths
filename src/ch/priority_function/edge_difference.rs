use super::PriorityFunction;
use crate::{
    ch::Shortcut,
    graphs::{Graph, VertexId},
};

pub struct EdgeDifference {}

impl Default for EdgeDifference {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeDifference {
    #[allow(unused_variables)]
    pub fn new() -> Self {
        Self {}
    }
}

impl PriorityFunction for EdgeDifference {
    #[allow(unused_variables)]
    fn priority(&self, vertex: VertexId, graph: &Box<dyn Graph>, shortcuts: &Vec<Shortcut>) -> i32 {
        shortcuts.len() as i32
            - graph.in_edges(vertex).len() as i32
            - graph.out_edges(vertex).len() as i32
    }

    #[allow(unused_variables)]
    fn update(&mut self, vertex: VertexId, graph: &Box<dyn Graph>) {}

    fn initialize(&mut self, _graph: &Box<dyn Graph>) {}
}
