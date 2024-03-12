use indicatif::ProgressBar;

use crate::{
    ch::{ch_queue::queue::CHQueue, shortcut::Shortcut},
    graphs::{graph::Graph, types::VertexId},
};

use super::Contractor;

pub struct SerialContractor {
    graph: Graph,
    priority_functions: String,
}

impl Contractor for SerialContractor {
    /// Generates contraction hierarchy where one vertex at a time is contracted.
    fn contract(mut self) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        let mut shortcuts = Vec::new();
        let mut levels = Vec::new();
        let mut queue = CHQueue::new(&self.graph, self.priority_functions.as_str());

        let bar = ProgressBar::new(self.graph.number_of_vertices() as u64);
        while let Some((vertex, vertex_shortcuts)) = queue.pop(&self.graph) {
            vertex_shortcuts.iter().for_each(|shortcut| {
                self.graph.add_edge(&shortcut.edge);
            });
            shortcuts.extend(vertex_shortcuts);

            self.graph.remove_vertex(vertex);
            levels.push(vec![vertex]);

            bar.inc(1);
        }
        bar.finish();

        (shortcuts, levels)
    }
}

impl SerialContractor {
    pub fn new(graph: &Graph, priority_functions: &str) -> Self {
        let graph = graph.clone();
        let priority_functions = priority_functions.into();

        SerialContractor {
            graph,
            priority_functions,
        }
    }
}
