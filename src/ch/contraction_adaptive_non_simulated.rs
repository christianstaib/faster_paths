use ahash::{HashMap, HashMapExt, HashSet};
use indicatif::ProgressBar;
use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    ch::{
        directed_contracted_graph::DirectedContractedGraph,
        helpers::generate_directed_contracted_graph, Shortcut,
    },
    classical_search::dijkstra::Dijkstra,
    graphs::{
        edge::{Edge, WeightedEdge},
        graph_functions::{all_edges, hitting_set, random_paths},
        reversible_hash_graph::ReversibleHashGraph,
        reversible_vec_graph::ReversibleVecGraph,
        vec_graph::VecGraph,
        Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub fn contract_adaptive_non_simulated_all_in(graph: &dyn Graph) -> DirectedContractedGraph {
    let graph_copy = ReversibleVecGraph::from_edges(&all_edges(graph));
    let dijkstra = Dijkstra {
        graph: Box::new(graph_copy),
    };
    let paths = random_paths(&dijkstra, 5_000, graph.number_of_vertices());
    let hitting_set = hitting_set(&paths, graph.number_of_vertices()).0;
    let landmarks = Landmarks::for_vertices(&hitting_set, graph);

    println!("copying base graph");
    let base_graph = VecGraph::from_edges(&all_edges(graph));

    println!("switching graph represenation");
    let mut graph = ReversibleHashGraph::from_edges(&all_edges(graph));

    let mut levels = Vec::new();

    println!("starting actual contraction");
    let mut all_shortcuts: HashMap<Edge, Shortcut> = HashMap::new();

    let mut remaining_vertices: HashSet<VertexId> = (0..graph.number_of_vertices()).collect();
    let bar = ProgressBar::new(remaining_vertices.len() as u64);

    while let Some(vertex) = get_next_vertex(&graph, &mut remaining_vertices) {
        // generating shortcuts
        let shortcuts = generate_all_shortcuts(&graph, &landmarks, vertex, &all_shortcuts);

        // adding shortcuts to graph and all_shortcuts
        shortcuts.iter().for_each(|shortcut| {
            graph.set_edge(&shortcut.edge);
        });

        shortcuts.into_iter().for_each(|shortcut| {
            all_shortcuts.insert(shortcut.edge.unweighted(), shortcut);
        });

        // removing graph
        graph.remove_vertex(vertex);

        levels.push(vec![vertex]);

        bar.inc(1);
    }
    bar.finish();

    let all_shortcuts = all_shortcuts.into_values().collect_vec();
    generate_directed_contracted_graph(base_graph, &all_shortcuts, &levels)
}

fn generate_all_shortcuts(
    graph: &dyn Graph,
    heuristic: &dyn Heuristic,
    vertex: u32,
    all_shortcuts: &HashMap<Edge, Shortcut>,
) -> Vec<Shortcut> {
    let in_edges = graph.in_edges(vertex).collect_vec();
    let out_edges = graph.out_edges(vertex).collect_vec();

    let shortcuts: Vec<_> = in_edges
        .iter()
        .cartesian_product(out_edges.iter())
        .par_bridge()
        .filter_map(|(in_edge, out_edge)| {
            if in_edge.tail() == out_edge.head() {
                return None;
            }

            let edge = WeightedEdge::new(
                in_edge.tail(),
                out_edge.head(),
                in_edge.weight() + out_edge.weight(),
            )
            .unwrap();

            if let Some(current_shortcut) = all_shortcuts.get(&edge.unweighted()) {
                let current_shortcut_weight = current_shortcut.edge.weight();
                if edge.weight() >= current_shortcut_weight {
                    return None;
                }
            }

            if !heuristic.respects_upper_bound(&edge) {
                return None;
            }

            if let Some(current_weight) = graph.get_edge_weight(&edge.unweighted()) {
                if edge.weight() >= current_weight {
                    return None;
                }
            }

            let shortcut = Shortcut { edge, vertex };

            Some(shortcut)
        })
        .collect();
    shortcuts
}

fn get_next_vertex(
    graph: &dyn Graph,
    remaining_vertices: &mut HashSet<VertexId>,
) -> Option<VertexId> {
    let min_vertex = *remaining_vertices.par_iter().min_by_key(|&&vertex| {
        (graph.in_edges(vertex).len() as i32 * graph.out_edges(vertex).len() as i32)
            - (graph.in_edges(vertex).len() as i32)
            - graph.out_edges(vertex).len() as i32
    })?;
    remaining_vertices.remove(&min_vertex);
    Some(min_vertex)
}
