use super::{
    contracted_graph::DirectedContractedGraph, contractor::helpers::partition_by_levels, Shortcut,
};
use crate::graphs::{vec_graph::VecGraph, Graph};

pub fn generate_directed_contracted_graph(
    mut base_graph: VecGraph,
    shortcuts: &[Shortcut],
    levels: Vec<Vec<u32>>,
) -> DirectedContractedGraph {
    for shortcut in shortcuts.iter() {
        base_graph.set_edge(&shortcut.edge);
    }

    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &levels);

    let shortcuts = shortcuts
        .iter()
        .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
        .collect();

    DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts,
        levels,
    }
}
