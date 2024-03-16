use indicatif::ProgressBar;

use crate::{
    ch::{queue::CHQueue, Shortcut},
    graphs::{graph::Graph, VertexId},
};

use super::Contractor;

pub struct SerialContractor {
    priority_functions: String,
}

impl Contractor for SerialContractor {
    /// Generates contraction hierarchy where one vertex at a time is contracted.
    fn contract(&self, graph: &Graph) -> (Vec<Shortcut>, Vec<Vec<VertexId>>) {
        let mut graph = graph.clone();
        let mut shortcuts = Vec::new();
        let mut levels = Vec::new();
        let mut queue = CHQueue::new(&graph, self.priority_functions.as_str());

        let bar = ProgressBar::new(graph.number_of_vertices() as u64);
        while let Some((vertex, vertex_shortcuts)) = queue.pop(&graph) {
            vertex_shortcuts.iter().for_each(|shortcut| {
                graph.add_edge(&shortcut.edge);
            });
            shortcuts.extend(vertex_shortcuts);

            graph.remove_vertex(vertex);
            levels.push(vec![vertex]);

            bar.inc(1);
        }
        bar.finish();

        (shortcuts, levels)
    }
}

impl SerialContractor {
    pub fn new(priority_functions: &str) -> Self {
        let priority_functions = priority_functions.into();

        SerialContractor { priority_functions }
    }
}
