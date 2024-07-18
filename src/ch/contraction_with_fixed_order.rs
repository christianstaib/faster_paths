use ahash::{HashMap, HashMapExt};
use indicatif::ProgressIterator;
use itertools::Itertools;

use crate::{
    ch::{
        contractor::contraction_helper::{ShortcutGenerator, ShortcutGeneratorWithWittnessSearch},
        directed_contracted_graph::DirectedContractedGraph,
        helpers::generate_directed_contracted_graph,
        Shortcut,
    },
    graphs::{
        edge::Edge, graph_functions::all_edges, reversible_vec_graph::ReversibleVecGraph,
        vec_graph::VecGraph, Graph, VertexId,
    },
};

pub fn contract_with_fixed_order(
    graph: &dyn Graph,
    level_to_vertices_map: &Vec<Vec<VertexId>>,
) -> DirectedContractedGraph {
    let mut working_graph = ReversibleVecGraph::from_edges(&all_edges(graph));
    let graph = VecGraph::from_edges(&all_edges(graph));

    let mut shortcuts: HashMap<Edge, Shortcut> = HashMap::new();

    println!("start contracting");

    for &vertex in level_to_vertices_map
        .iter()
        .flatten()
        // .rev()
        .progress_count(graph.number_of_vertices() as u64)
    {
        let vertex_shortcuts = ShortcutGeneratorWithWittnessSearch { max_hops: 16 }
            .get_shortcuts(&working_graph, vertex);

        vertex_shortcuts.into_iter().for_each(|shortcut| {
            let current_weight = working_graph
                .get_edge_weight(&shortcut.edge.unweighted())
                .unwrap_or(u32::MAX);
            if shortcut.edge.weight() < current_weight {
                working_graph.set_edge(&shortcut.edge);
                shortcuts.insert(shortcut.edge.unweighted(), shortcut);
            }
        });

        working_graph.remove_vertex(vertex);
    }

    let shortcuts = shortcuts.into_values().collect_vec();
    generate_directed_contracted_graph(graph, &shortcuts, level_to_vertices_map)
}
