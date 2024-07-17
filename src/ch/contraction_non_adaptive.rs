use ahash::{HashMap, HashMapExt};
use indicatif::ProgressBar;
use itertools::Itertools;

use crate::{
    ch::{
        contractor::contraction_helper::{ShortcutGenerator, ShortcutGeneratorWithWittnessSearch},
        directed_contracted_graph::DirectedContractedGraph,
        helpers::generate_directed_contracted_graph,
        Shortcut,
    },
    graphs::{edge::Edge, graph_functions::all_edges, vec_graph::VecGraph, Graph, VertexId},
};

pub fn contract_non_adaptive(
    graph: &dyn Graph,
    level_to_vertices_mal: &[Vec<VertexId>],
) -> DirectedContractedGraph {
    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    let mut base_graph = VecGraph::from_edges(&all_edges(graph));

    let mut shortcuts: HashMap<Edge, Shortcut> = HashMap::new();
    let mut levels = Vec::new();

    println!("start contracting");
    let bar = ProgressBar::new(base_graph.number_of_vertices() as u64);

    for &vertex in level_to_vertices_mal.iter().flatten().rev() {
        let vertex_shortcuts =
            ShortcutGeneratorWithWittnessSearch { max_hops: 16 }.get_shortcuts(&base_graph, vertex);

        vertex_shortcuts.into_iter().for_each(|shortcut| {
            let current_weight = base_graph
                .get_edge_weight(&shortcut.edge.unweighted())
                .unwrap_or(u32::MAX);
            if shortcut.edge.weight() < current_weight {
                base_graph.set_edge(&shortcut.edge);
                shortcuts.insert(shortcut.edge.unweighted(), shortcut);
            }
        });

        base_graph.remove_vertex(vertex);

        levels.push(vec![vertex]);
        bar.inc(1);
    }

    let shortcuts = shortcuts.into_values().collect_vec();
    generate_directed_contracted_graph(vec_graph, &shortcuts, levels)
}
