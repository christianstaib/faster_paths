use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    process::exit,
    time::Instant,
    usize,
};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use clap::Parser;
use faster_paths::{
    ch::{
        all_in_preprocessor::AllInPrerocessor, preprocessor::Preprocessor,
        shortcut_replacer::fast_shortcut_replacer::FastShortcutReplacer,
    },
    dijkstra_data::dijkstra_data_vec::DijkstraDataVec,
    graphs::{
        edge::DirectedWeightedEdge,
        graph_factory::GraphFactory,
        graph_functions::{add_edge_bidrectional, all_edges},
        path::{PathFinding, ShortestPathTestCase},
        reversible_hash_graph::ReversibleHashGraph,
        reversible_vec_graph::ReversibleVecGraph,
        Graph, VertexId, Weight,
    },
    hl::{hub_graph::HubGraph, label::Label, label_entry::LabelEntry},
    simple_algorithms::dijkstra::Dijkstra,
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;

use rand::prelude::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    infile: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    tests: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    outfile: PathBuf,
}

fn main() {
    let args = Args::parse();

    // let graph = get_small_graph();
    // let order = vec![11, 1, 7, 5, 2, 10, 6, 9, 3, 8, 4];
    // let label = get_out_label(1, &graph, &order);

    // println!();
    // for entry in label.entries.iter() {
    //     println!("{}", entry.vertex);
    // }
    // exit(0);

    println!("loading test cases");
    let reader = BufReader::new(File::open(&args.tests).unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.infile);

    let mut order = (0..graph.number_of_vertices()).collect_vec();
    order.shuffle(&mut rand::thread_rng());

    let hub_graph = get_hl(&graph, &order);

    test_cases
        .par_iter()
        .take(1_000)
        .progress()
        .for_each(|test_case| {
            let weight = hub_graph.shortest_path_weight(&test_case.request);

            if weight != test_case.weight {
                println!("err soll {:?}, ist {:?}", test_case.weight, weight);
            }
        });

    println!("all {} tests passed", test_cases.len());
}

fn get_hl(graph: &dyn Graph, order: &[u32]) -> HubGraph {
    let dijkstra = Dijkstra::new(graph);

    let forward_labels: Vec<_> = (0..graph.number_of_vertices())
        .progress()
        .par_bridge()
        .map(|source| {
            let data = dijkstra.single_source(source);
            get_label(source, &data, &order)
        })
        .collect();

    let reverse_labels: Vec<_> = forward_labels.clone();
    // let reverse_labels = (0..graph.number_of_vertices())
    //     .progress()
    //     .par_bridge()
    //     .map(|source| {
    //         let data = dijkstra.single_target(source);
    //         get_label(source, &data, &order)
    //     })
    //     .collect();

    HubGraph {
        forward_labels,
        reverse_labels,
        shortcut_replacer: FastShortcutReplacer {
            shortcuts: HashMap::new(),
        },
    }
}

fn get_label(vertex: VertexId, data: &DijkstraDataVec, order: &[u32]) -> Label {
    let mut children = vec![Vec::new(); data.vertices.len()];

    for (vertex, entry) in data.vertices.iter().enumerate() {
        if let Some(predecessor) = entry.predecessor {
            children[predecessor as usize].push(vertex);
        }
    }

    let mut stack = vec![vertex as usize];

    let mut label = Label::new(vertex);
    while let Some(current) = stack.pop() {
        let mut current_children = std::mem::take(&mut children[current as usize]);

        // println!();
        // println!("looking at {}", current);
        while let Some(child) = current_children.pop() {
            if order[child] > order[current] {
                // println!("including edge {} -> {}", current, child);
                stack.push(child);
                label.entries.push(LabelEntry {
                    vertex: child as VertexId,
                    predecessor: Some(current as VertexId),
                    weight: data.vertices[child].weight.unwrap(),
                });
            } else {
                // println!("not including edge {} -> {}", current, child);
                current_children.extend(std::mem::take(&mut children[child]));
            }
        }
    }

    label.entries.sort_unstable_by_key(|entry| entry.vertex);

    label
}
