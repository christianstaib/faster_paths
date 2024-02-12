use crate::{
    ch::contraction_helper::ContractionHelper,
    graphs::{graph::Graph, types::VertexId},
};

use super::queue::PriorityTerm;

pub struct EdgeDifferencePriority {}

impl PriorityTerm for EdgeDifferencePriority {
    fn priority(&self, vertex: VertexId, graph: &Graph) -> i32 {
        let shortcut_generator = ContractionHelper::new(graph, 5);
        let shortcuts = shortcut_generator.generate_shortcuts(vertex);

        let number_of_edges =
            graph.in_edges[vertex as usize].len() + graph.out_edges[vertex as usize].len();

        shortcuts.len() as i32 - number_of_edges as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, v: u32, graph: &Graph) {}
}

#[allow(unused_variables)]
impl EdgeDifferencePriority {
    pub fn new(graph: &Graph) -> Self {
        Self {}
    }
}
