use super::{
    contractor::{
        contraction_helper::{
            partition_by_levels, ShortcutGeneratorWithHeuristic,
            ShortcutGeneratorWithWittnessSearch,
        },
        serial_witness_search_contractor::SerialAdaptiveSimulatedContractor,
    },
    priority_function::decode_function,
    Shortcut,
};
use crate::{
    ch::contracted_graph::DirectedContractedGraph,
    graphs::{graph_functions::to_vec_graph, vec_graph::VecGraph, Graph},
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub fn contract_adaptive_simulated_with_witness(graph: &dyn Graph) -> DirectedContractedGraph {
    let vec_graph = to_vec_graph(graph);
    let priority_terms = decode_function("E:1_D:1_C:1");

    let shortcut_generator = ShortcutGeneratorWithWittnessSearch { max_hops: 16 };
    let shortcut_generator = Box::new(shortcut_generator);
    let mut contractor = SerialAdaptiveSimulatedContractor::new(priority_terms, shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    get_ch_stateless(vec_graph, &shortcuts, &levels)
}

pub fn contract_adaptive_simulated_with_landmarks(graph: &dyn Graph) -> DirectedContractedGraph {
    let vec_graph = to_vec_graph(&*graph);
    let priority_terms = decode_function("E:1_D:1_C:1");

    let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(10, &*graph));
    let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };
    let shortcut_generator = Box::new(shortcut_generator);

    let mut contractor = SerialAdaptiveSimulatedContractor::new(priority_terms, shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    get_ch_stateless(vec_graph, &shortcuts, &levels)
}

pub fn get_ch_stateless(
    mut base_graph: VecGraph,
    shortcuts: &[Shortcut],
    levels: &[Vec<u32>],
) -> DirectedContractedGraph {
    for shortcut in shortcuts.iter() {
        base_graph.set_edge(&shortcut.edge);
    }

    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, levels);

    let shortcuts = shortcuts
        .iter()
        .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
        .collect();

    let directed_contracted_graph = DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts,
        levels: levels.to_vec(),
    };

    directed_contracted_graph
}
