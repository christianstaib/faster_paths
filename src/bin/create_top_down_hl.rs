use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::{Arc, Mutex},
    usize,
};

use ahash::{HashMap, HashMapExt, HashSetExt};
use clap::Parser;
use faster_paths::{
    ch::{
        shortcut_replacer::{
            self,
            fast_shortcut_replacer::{self, FastShortcutReplacer},
            slow_shortcut_replacer::SlowShortcutReplacer,
        },
        Shortcut,
    },
    dijkstra_data::{dijkstra_data_vec::DijkstraDataVec, DijkstraData},
    graphs::{
        edge::{DirectedEdge, DirectedWeightedEdge},
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathTestCase},
        Graph, VertexId,
    },
    hl::{hub_graph::HubGraph, label::Label, label_entry::LabelEntry},
    simple_algorithms::dijkstra::Dijkstra,
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;

use rand::prelude::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
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

    println!("loading test cases");
    let reader = BufReader::new(File::open(&args.tests).unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.infile);

    let mut order = (0..graph.number_of_vertices()).collect_vec();
    order.shuffle(&mut rand::thread_rng());

    let hub_graph = get_hl(&graph, &order);

    let writer = BufWriter::new(File::create("hl_test.bincode").unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();

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
    let shortcuts: Arc<Mutex<HashMap<DirectedEdge, VertexId>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let forward_labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, label_shortcuts) = get_out_label(vertex, graph, order);

            if let Ok(mut shortcuts) = shortcuts.lock() {
                for shortcut in label_shortcuts {
                    shortcuts.insert(shortcut.edge.unweighted(), shortcut.vertex);
                }
            }

            label
        })
        .collect();

    let reverse_labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, label_shortcuts) = get_in_label(vertex, graph, order);

            if let Ok(mut shortcuts) = shortcuts.lock() {
                for shortcut in label_shortcuts {
                    shortcuts.insert(shortcut.edge.unweighted(), shortcut.vertex);
                }
            }

            label
        })
        .collect();

    let shortcuts = Vec::new();
    let slow_shortcut_replacer = SlowShortcutReplacer::new(&shortcuts);
    let shortcut_replacer = FastShortcutReplacer::new(&slow_shortcut_replacer);

    HubGraph {
        forward_labels,
        reverse_labels,
        shortcut_replacer,
    }
}

fn shortests_path_tree(data: &DijkstraDataVec) -> Vec<Vec<VertexId>> {
    let mut search_tree = vec![Vec::new(); data.vertices.len()];

    for (child, entry) in data.vertices.iter().enumerate() {
        if let Some(parent) = entry.predecessor {
            search_tree[parent as usize].push(child as VertexId);
        }
    }

    search_tree
}

fn get_out_label(vertex: VertexId, graph: &dyn Graph, order: &[u32]) -> (Label, Vec<Shortcut>) {
    let dijkstra = Dijkstra::new(graph);
    let data = dijkstra.single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

fn get_in_label(vertex: VertexId, graph: &dyn Graph, order: &[u32]) -> (Label, Vec<Shortcut>) {
    let dijkstra = Dijkstra::new(graph);
    let data = dijkstra.single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

fn get_label_from_data(
    vertex: VertexId,
    data: &DijkstraDataVec,
    order: &[u32],
) -> (Label, Vec<Shortcut>) {
    let mut shortest_path_tree = shortests_path_tree(data);
    let mut shortcuts = Vec::new();

    let mut stack = vec![vertex as usize];

    let mut label = Label::new(vertex);
    while let Some(parent) = stack.pop() {
        let mut children = std::mem::take(&mut shortest_path_tree[parent]);

        while let Some(child) = children.pop() {
            if order[child as usize] > order[parent] {
                stack.push(child as usize);
                label.entries.push(LabelEntry {
                    vertex: child,
                    predecessor: Some(parent as VertexId),
                    weight: data.vertices[child as usize].weight.unwrap(),
                });
            } else {
                for &child_child in std::mem::take(&mut shortest_path_tree[child as usize]).iter() {
                    children.push(child_child);
                    let weight = data.vertices[child_child as usize].weight.unwrap()
                        - data.vertices[parent].weight.unwrap();
                    let shortcut = Shortcut {
                        edge: DirectedWeightedEdge::new(parent as VertexId, child_child, weight)
                            .unwrap(),
                        vertex: child_child,
                    };
                    shortcuts.push(shortcut);
                }
            }
        }
    }

    label.entries.sort_unstable_by_key(|entry| entry.vertex);

    (label, shortcuts)
}
