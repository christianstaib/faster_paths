use indicatif::ProgressIterator;

use itertools::Itertools;
use rand::prelude::*;

use super::{
    contractor::{
        contraction_helper::{
            ShortcutGenerator, ShortcutGeneratorWithHeuristic, ShortcutGeneratorWithWittnessSearch,
        },
        serial_witness_search_contractor::SerialAdaptiveSimulatedContractor,
    },
    helpers::generate_directed_contracted_graph,
    priority_function::decode_function,
};
use crate::{
    ch::directed_contracted_graph::DirectedContractedGraph,
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
    let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(100, graph));
    let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };

    let mut all_avg_dif = Vec::new();

    // shuffle vertices for smooth progress bar
    let mut vertices = (0..graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    println!("start predicting");
    for vertex in vertices.into_iter().progress() {
        let mut predicted = Vec::new();
        for _ in 0..10 {
            predicted.push(shortcut_generator.get_shortcuts_predicited(graph, vertex));
        }
        let actual = shortcut_generator.get_shortcuts(graph, vertex).len() as usize;

        let diff_percent = predicted
            .into_iter()
            .map(|x| ((actual as i32 - x as i32).abs() as f32 / actual as f32) * 100.0)
            .collect_vec();

        let mut diff = diff_percent.iter().sum::<f32>() / diff_percent.len() as f32;

        if diff.is_nan() {
            diff = 0.0;
        }
        all_avg_dif.push(diff);
        let mut diff = all_avg_dif.iter().sum::<f32>() / all_avg_dif.len() as f32;

        println!("avg diff {:?}", diff);
    }

    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    let priority_terms = decode_function("E:1_D:1_C:1");
    let mut contractor =
        SerialAdaptiveSimulatedContractor::new(priority_terms, &shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    generate_directed_contracted_graph(vec_graph, &shortcuts, levels)
}
