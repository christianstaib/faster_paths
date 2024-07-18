use super::{
    contractor::helpers::partition_by_levels, directed_contracted_graph::DirectedContractedGraph,
    Shortcut,
};
use crate::graphs::{vec_graph::VecGraph, Graph};

pub fn generate_directed_contracted_graph(
    mut base_graph: VecGraph,
    shortcuts: &Vec<Shortcut>,
    level_to_vertices_map: &Vec<Vec<u32>>,
) -> DirectedContractedGraph {
    for shortcut in shortcuts.iter() {
        base_graph.set_edge(&shortcut.edge);
    }

    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &level_to_vertices_map);

    let shortcuts = shortcuts
        .iter()
        .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
        .collect();

    DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts,
        level_to_vertices_map: level_to_vertices_map.clone(),
    }
}
