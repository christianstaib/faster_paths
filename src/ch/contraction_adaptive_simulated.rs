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

fn median_and_average_deviation_from_ground_truth(vec: &Vec<i32>, truth: i32) -> (f64, f64) {
    // Step 1: Calculate the absolute deviations
    let deviations: Vec<f64> = vec.iter().map(|&x| (x - truth).abs() as f64).collect();

    // Step 2: Sort the deviations for median calculation
    let mut sorted_deviations = deviations.clone();
    sorted_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Step 3: Find the median of the deviations
    let median_deviation = if sorted_deviations.len() % 2 == 0 {
        let mid = sorted_deviations.len() / 2;
        (sorted_deviations[mid - 1] + sorted_deviations[mid]) / 2.0
    } else {
        sorted_deviations[sorted_deviations.len() / 2]
    };

    // Step 4: Calculate the average of the deviations
    let total_deviation: f64 = deviations.iter().sum();
    let average_deviation = total_deviation / deviations.len() as f64;

    // Step 5: Calculate the deviations as a percentage of the ground truth
    let median_deviation_percentage = (median_deviation / truth as f64) * 100.0;
    let average_deviation_percentage = (average_deviation / truth as f64) * 100.0;

    (median_deviation_percentage, average_deviation_percentage)
}

pub fn contract_adaptive_simulated_with_landmarks(graph: &dyn Graph) -> DirectedContractedGraph {
    let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(100, graph));
    let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };

    // shuffle vertices for smooth progress bar
    let mut vertices = (0..graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    println!("start predicting");
    for vertex in vertices.into_iter().progress() {
        let mut predicted_edge_difference = Vec::new();
        for _ in 0..10 {
            predicted_edge_difference
                .push(shortcut_generator.get_edge_difference_predicited(graph, vertex));
        }
        let actual_edge_difference = shortcut_generator.get_shortcuts(graph, vertex).len() as i32
            - graph.out_edges(vertex).len() as i32
            - graph.in_edges(vertex).len() as i32;

        let average_predicted_edge_difference = predicted_edge_difference
            .iter()
            .map(|&x| x as f32)
            .sum::<f32>()
            / predicted_edge_difference.len() as f32;

        println!(
            "avg: {:?}, actual: {}",
            average_predicted_edge_difference, actual_edge_difference
        );
    }

    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    let priority_terms = decode_function("E:1_D:1_C:1");
    let mut contractor =
        SerialAdaptiveSimulatedContractor::new(priority_terms, &shortcut_generator);

    let (shortcuts, levels) = contractor.contract(graph);
    generate_directed_contracted_graph(vec_graph, &shortcuts, levels)
}
