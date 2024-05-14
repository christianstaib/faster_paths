use super::{
    contractor::{
        contraction_helper::{ShortcutGeneratorWithHeuristic, ShortcutGeneratorWithWittnessSearch},
        helpers::partition_by_levels,
        serial_witness_search_contractor::SerialAdaptiveSimulatedContractor,
    },
    priority_function::decode_function,
    Shortcut,
};
use crate::{
    ch::contracted_graph::DirectedContractedGraph,
    graphs::{graph_functions::all_edges, vec_graph::VecGraph, Graph},
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub fn contract_adaptive_simulated_with_witness(graph: &dyn Graph) -> DirectedContractedGraph {
    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    let priority_terms = decode_function("E:1_D:1_C:1");

    let shortcut_generator = ShortcutGeneratorWithWittnessSearch { max_hops: 16 };
    let mut contractor =
        SerialAdaptiveSimulatedContractor::new(priority_terms, &shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    generate_directed_contracted_graph(vec_graph, &shortcuts, levels)
}

pub fn contract_adaptive_simulated_with_landmarks(graph: &dyn Graph) -> DirectedContractedGraph {
    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    let priority_terms = decode_function("E:1_D:1_C:1");

    let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(10, graph));
    let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };
    let mut contractor =
        SerialAdaptiveSimulatedContractor::new(priority_terms, &shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    generate_directed_contracted_graph(vec_graph, &shortcuts, levels)
}

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
