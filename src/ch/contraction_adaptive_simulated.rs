use std::{
    collections::BinaryHeap,
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

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
    ch::{
        ch_priority_element::ChPriorityElement, directed_contracted_graph::DirectedContractedGraph,
        Shortcut,
    },
    graphs::{
        edge::DirectedEdge, graph_functions::all_edges, reversible_hash_graph::ReversibleHashGraph,
        reversible_vec_graph::ReversibleVecGraph, vec_graph::VecGraph, Graph, VertexId,
    },
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
    let hitting_set: HashSet<VertexId> = HashSet::new();

    let mut work_graph = ReversibleVecGraph::from_edges(&all_edges(graph));
    let heuristic: Box<dyn Heuristic> = Box::new(Landmarks::new(100, &work_graph));
    let shortcut_generator = ShortcutGeneratorWithHeuristic { heuristic };

    // shuffle vertices for smooth progress bar
    let mut vertices = (0..work_graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    println!("initalizing queue");
    let mut queue: BinaryHeap<_> = vertices
        .par_iter()
        .progress()
        .map(|&vertex| {
            let mut priority =
                shortcut_generator.get_edge_difference_predicited(&work_graph, vertex);
            if hitting_set.contains(&vertex) {
                priority += i32::MAX / 2;
            }
            ChPriorityElement { vertex, priority }
        })
        .collect();

    let mut level_to_verticies_map = Vec::new();
    let mut shortcuts: HashMap<DirectedEdge, Shortcut> = HashMap::new();

    let mut writer = BufWriter::new(File::create("time.csv").unwrap());
    writeln!(
        writer,
        "pop,add shortcuts,gen shortcuts,remove vertex,update neighbors"
    )
    .unwrap();

    println!("start contracting");
    let bar = ProgressBar::new(work_graph.number_of_vertices() as u64);
    let mut start = Instant::now();

    let update_intervall = graph.number_of_vertices() / 10;
    while let Some(state) = queue.pop() {
        let duration_pop = start.elapsed();
        start = Instant::now();

        // let neighbors = neighbors(state.vertex, &work_graph);

        let mut vertex_shortcuts = shortcut_generator.get_shortcuts(&work_graph, state.vertex);
        let duration_gen_shortcuts = start.elapsed();
        start = Instant::now();

        vertex_shortcuts = vertex_shortcuts
            .into_par_iter()
            .flat_map(|shortcut| {
                let current_weight = work_graph
                    .get_edge_weight(&shortcut.edge.unweighted())
                    .unwrap_or(u32::MAX);
                if shortcut.edge.weight() >= current_weight {
                    return None;
                }
                Some(shortcut)
            })
            .collect();

        vertex_shortcuts.into_iter().for_each(|shortcut| {
            work_graph.set_edge(&shortcut.edge);
            shortcuts.insert(shortcut.edge.unweighted(), shortcut);
        });

        let duration_add_shortcuts = start.elapsed();
        start = Instant::now();

        work_graph.remove_vertex(state.vertex);

        let duration_remove_vertex = start.elapsed();
        start = Instant::now();

        if bar.position() % update_intervall as u64 == 0 {
            queue = queue
                .into_par_iter()
                .map(|mut state| {
                    // if neighbors.contains(&state.vertex) {
                    state.priority =
                        shortcut_generator.get_edge_difference_predicited(graph, state.vertex);
                    if hitting_set.contains(&state.vertex) {
                        state.priority += i32::MAX / 2;
                    }
                    // }
                    state
                })
                .collect();
        }
        let duration_update_neighbors = start.elapsed();

        writeln!(
            writer,
            "{},{},{},{},{}",
            duration_pop.as_secs_f64(),
            duration_gen_shortcuts.as_secs_f64(),
            duration_add_shortcuts.as_secs_f64(),
            duration_remove_vertex.as_secs_f64(),
            duration_update_neighbors.as_secs_f64()
        )
        .unwrap();
        start = Instant::now();

        level_to_verticies_map.push(vec![state.vertex]);
        bar.inc(1);
    }
    bar.finish();
    writer.flush().unwrap();

    let (shortcuts, levels) = (
        shortcuts.into_values().collect_vec(),
        level_to_verticies_map,
    );

    let vec_graph = VecGraph::from_edges(&all_edges(graph));
    generate_directed_contracted_graph(vec_graph, &shortcuts, levels)
}
