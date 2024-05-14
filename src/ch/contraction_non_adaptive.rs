use ahash::{HashMap, HashMapExt};
use indicatif::ProgressBar;
use itertools::Itertools;

use crate::{
    ch::{
        contracted_graph::DirectedContractedGraph,
        contractor::contraction_helper::{
            partition_by_levels, ShortcutGenerator, ShortcutGeneratorWithWittnessSearch,
        },
        Shortcut,
    },
    graphs::{
        edge::DirectedEdge, graph_functions::all_edges, reversible_vec_graph::ReversibleVecGraph,
        Graph, VertexId,
    },
};

pub fn contract_non_adaptive(graph: &dyn Graph, order: &[VertexId]) -> DirectedContractedGraph {
    let mut base_graph = ReversibleVecGraph::from_edges(&all_edges(graph));

    let mut shortcuts: HashMap<DirectedEdge, Shortcut> = HashMap::new();
    let mut levels = Vec::new();

    println!("start contracting");
    let bar = ProgressBar::new(base_graph.number_of_vertices() as u64);

    for &vertex in order.iter().rev() {
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

    println!("assing shortcuts to base graph");
    let edges = shortcuts
        .values()
        .map(|shortcut| shortcut.edge.clone())
        .collect_vec();
    base_graph.set_edges(&edges);

    println!("creating upward and downward_graph");
    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &levels);

    println!("generatin shortcut lookup map");
    let shortcuts = shortcuts
        .values()
        .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
        .collect();

    

    DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts,
        levels,
    }
}
