use std::usize;

use itertools::Itertools;

use crate::{
    ch::{
        contracted_graph::ContractedGraph,
        shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer,
    },
    graphs::{
        edge::DirectedWeightedEdge, graph_functions::to_vec_graph, vec_graph::VecGraph, Graph,
    },
    heuristics::{landmarks::Landmarks, none_heuristic::NoneHeuristic, Heuristic},
};

use super::{
    contractor::{
        contraction_helper::{ShortcutGeneratorWithHeuristic, ShortcutGeneratorWithWittnessSearch},
        serial_witness_search_contractor::SerialWitnessSearchContractor,
        Contractor,
    },
    priority_function::decode_function,
};

pub struct Preprocessor {
    contractor: Box<dyn Contractor>,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new_wittness_search()
    }
}

impl Preprocessor {
    pub fn new_wittness_search() -> Self {
        let shortcut_generator = ShortcutGeneratorWithWittnessSearch { max_hops: 16 };
        let shortcut_generator = Box::new(shortcut_generator);
        Preprocessor {
            contractor: Box::new(SerialWitnessSearchContractor::new(
                decode_function("E:1_D:1_C:1"),
                shortcut_generator,
            )),
        }
    }

    pub fn new_all_in() -> Self {
        let heuristic: Box<dyn Heuristic> = Box::new(NoneHeuristic {});
        let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };
        let shortcut_generator = Box::new(shortcut_generator);
        Preprocessor {
            contractor: Box::new(SerialWitnessSearchContractor::new(
                decode_function("E:1_D:1_C:1"),
                shortcut_generator,
            )),
        }
    }

    pub fn new_landmark(graph: &dyn Graph) -> Self {
        let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(10, graph));
        let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };
        let shortcut_generator = Box::new(shortcut_generator);
        Preprocessor {
            contractor: Box::new(SerialWitnessSearchContractor::new(
                decode_function("E:1_D:1_C:1"),
                shortcut_generator,
            )),
        }
    }

    pub fn with_contractor(contractor: Box<dyn Contractor>) -> Self {
        Preprocessor { contractor }
    }

    pub fn get_ch(&mut self, graph: Box<dyn Graph>) -> ContractedGraph {
        let mut base_graph = to_vec_graph(&*graph);

        let (shortcuts, levels) = self.contractor.contract(graph);
        println!("fin contract");

        for shortcut in shortcuts.iter() {
            base_graph.set_edge(&shortcut.edge);
        }
        let (upward_graph, downward_graph) =
            removing_edges_violating_level_property(&base_graph, &levels);

        let shortcuts = shortcuts
            .iter()
            .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
            .collect_vec();

        let shortcut_replacer = SlowShortcutReplacer::new(&shortcuts);

        ContractedGraph {
            upward_graph,
            downward_graph,
            shortcut_replacer,
            levels,
        }
    }
}

pub fn removing_edges_violating_level_property(
    graph: &dyn Graph,
    levels: &Vec<Vec<u32>>,
) -> (VecGraph, VecGraph) {
    let mut vertex_to_level = vec![0; graph.number_of_vertices() as usize];
    for (level, level_list) in levels.iter().enumerate() {
        for &vertex in level_list.iter() {
            vertex_to_level[vertex as usize] = level;
        }
    }

    let edges: Vec<_> = (0..graph.number_of_vertices())
        .flat_map(|vertex| graph.out_edges(vertex))
        .collect();

    let upward_edges: Vec<_> = edges
        .iter()
        .filter(|edge| {
            vertex_to_level[edge.tail() as usize] <= vertex_to_level[edge.head() as usize]
        })
        .cloned()
        .collect();
    let upward_graph = VecGraph::from_edges(&upward_edges);

    let downward_edges: Vec<_> = edges
        .iter()
        .map(DirectedWeightedEdge::reversed)
        .filter(|edge| {
            vertex_to_level[edge.tail() as usize] <= vertex_to_level[edge.head() as usize]
        })
        .collect();
    let downard_graph = VecGraph::from_edges(&downward_edges);

    (upward_graph, downard_graph)
}
