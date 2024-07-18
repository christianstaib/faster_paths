use ahash::{HashMap, HashMapExt};
use indicatif::ProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    ch::{
        contractor::contraction_helper::{ShortcutGenerator, ShortcutGeneratorWithHeuristic},
        directed_contracted_graph::DirectedContractedGraph,
        helpers::generate_directed_contracted_graph,
        Shortcut,
    },
    graphs::{
        edge::Edge, graph_functions::all_edges, reversible_vec_graph::ReversibleVecGraph,
        vec_graph::VecGraph, Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

/// Contract `graph` with a fixed order defined by `level_to_vertices_map`.
///
/// The vertices of level 0 are in `level_to_vertices_map[0]`.
pub fn contract_with_fixed_order(
    graph: &dyn Graph,
    level_to_vertices_map: &Vec<Vec<VertexId>>,
) -> DirectedContractedGraph {
    let graph = VecGraph::from_edges(&all_edges(graph));

    let mut working_graph = ReversibleVecGraph::from_edges(&all_edges(&graph));
    let mut shortcuts: HashMap<Edge, Shortcut> = HashMap::new();

    let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(100, &working_graph));
    let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };

    println!("start contracting");
    for &vertex in level_to_vertices_map
        .iter()
        .flatten()
        .progress_count(graph.number_of_vertices() as u64)
    {
        let mut vertex_shortcuts = shortcut_generator.get_shortcuts(&working_graph, vertex);

        vertex_shortcuts = vertex_shortcuts
            .into_par_iter()
            .flat_map(|shortcut| {
                let current_weight = working_graph
                    .get_edge_weight(&shortcut.edge.unweighted())
                    .unwrap_or(u32::MAX);
                if shortcut.edge.weight() >= current_weight {
                    return None;
                }
                Some(shortcut)
            })
            .collect();

        vertex_shortcuts.into_iter().for_each(|shortcut| {
            working_graph.set_edge(&shortcut.edge);
            shortcuts.insert(shortcut.edge.unweighted(), shortcut);
        });

        working_graph.remove_vertex(vertex);
    }

    let shortcuts = shortcuts.into_values().collect();
    generate_directed_contracted_graph(graph, &shortcuts, level_to_vertices_map)
}
