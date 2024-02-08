use crate::{ch::contraction_helper::ContractionHelper, graph::Graph};

use super::queue::PriorityTerm;

pub struct EdgeDifferencePriority {}

impl PriorityTerm for EdgeDifferencePriority {
    fn priority(&self, v: u32, graph: &Graph) -> i32 {
        let shortcut_generator = ContractionHelper::new(graph, 10);
        let shortcuts = shortcut_generator.generate_shortcuts(v);

        let current_pairs = graph.in_edges[v as usize].len() + graph.out_edges[v as usize].len();

        shortcuts.len() as i32 - current_pairs as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, v: u32, graph: &Graph) {}
}

impl Default for EdgeDifferencePriority {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeDifferencePriority {
    pub fn new() -> Self {
        Self {}
    }
}
