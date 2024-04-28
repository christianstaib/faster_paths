use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
    usize,
};

use ahash::{HashMap, HashMapExt};
use clap::Parser;
use faster_paths::{
    classical_search::dijkstra::Dijkstra,
    dijkstra_data::dijkstra_data_vec::DijkstraDataVec,
    graphs::{
        edge::DirectedEdge,
        graph_factory::GraphFactory,
        graph_functions::{hitting_set, random_paths},
        path::{PathFinding, ShortestPathTestCase},
        Graph, VertexId,
    },
    hl::{
        hl_path_finding::HLPathFinder,
        hub_graph::DirectedHubGraph,
        label::{Label, LabelEntry},
    },
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

    let paths = random_paths(500_000, &graph);
    let mut hitting_set = hitting_set(&paths, graph.number_of_vertices());

    let mut not_hitting_set = (0..graph.number_of_vertices())
        .into_iter()
        .filter(|vertex| !hitting_set.contains(&vertex))
        .collect_vec();
    not_hitting_set.shuffle(&mut thread_rng());

    hitting_set.extend(not_hitting_set);
    hitting_set.reverse();

    // let mut order = (0..graph.number_of_vertices()).collect_vec();
    // order.shuffle(&mut rand::thread_rng());
    let order: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .map(|vertex| hitting_set.iter().position(|&x| x == vertex).unwrap() as u32)
        .collect();

    let n = 1_000;

    let start = Instant::now();
    order.par_iter().take(n).progress().for_each(|&vertex| {
        get_out_label(vertex, &graph, &order);
    });
    println!(
        "will take {:?} for whole graph",
        start.elapsed() / n as u32 * graph.number_of_vertices()
    );

    println!("generating hl");
    let hub_graph = get_hl(&graph, &order);
    let hub_graph_path_finder = HLPathFinder {
        hub_graph: &hub_graph,
    };

    let writer = BufWriter::new(File::create("hl_test.bincode").unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();

    test_cases
        .par_iter()
        .take(1_000)
        .progress()
        .for_each(|test_case| {
            // let forward_label = get_out_label(test_case.request.source(), &graph, &order).0;
            // let reverse_label = get_in_label(test_case.request.target(), &graph, &order).0;
            // let (weight, _, _) = HubGraph::overlap(&forward_label, &reverse_label).unwrap();
            // let weight = Some(weight);

            let weight = hub_graph_path_finder.shortest_path_weight(&test_case.request);
            assert_eq!(weight, test_case.weight);

            // let _path = hub_graph.shortest_path(&test_case.request);

            // if let Err(err) = validate_path(&graph, test_case, &_path) {
            //     panic!("top down hl wrong: {}", err);
            // }
        });

    println!("all {} tests passed", test_cases.len());
}

fn get_hl(graph: &dyn Graph, order: &[u32]) -> DirectedHubGraph {
    let shortcuts: Arc<Mutex<HashMap<DirectedEdge, VertexId>>> =
        Arc::new(Mutex::new(HashMap::new()));

    println!("generating forward labels");
    let forward_labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (mut label, label_shortcuts) = get_out_label(vertex, graph, order);
            label.entries.shrink_to_fit();

            shortcuts.lock().unwrap().extend(label_shortcuts);

            label
        })
        .collect();

    println!("generating reverse labels");
    // let reverse_labels: Vec<_> = (0..graph.number_of_vertices())
    //     .into_par_iter()
    //     .progress()
    //     .map(|vertex| {
    //         if vertex % 1_000 == 0 {
    //             println!("{}/{}", vertex, graph.number_of_vertices());
    //         }

    //         let (label, label_shortcuts) = get_in_label(vertex, graph, order);

    //         shortcuts.lock().unwrap().extend(label_shortcuts);

    //         label
    //     })
    //     .collect();
    let reverse_labels = forward_labels.clone();

    println!("getting shortcuts vec");
    let _shortcuts = shortcuts
        .lock()
        .unwrap()
        .to_owned()
        .into_iter()
        .collect_vec();

    DirectedHubGraph {
        forward_labels,
        reverse_labels,
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

fn get_out_label(
    vertex: VertexId,
    graph: &dyn Graph,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let dijkstra = Dijkstra::new(graph);
    let data = dijkstra.single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

fn get_in_label(
    vertex: VertexId,
    graph: &dyn Graph,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let dijkstra = Dijkstra::new(graph);
    let data = dijkstra.single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

fn get_label_from_data(
    vertex: VertexId,
    data: &DijkstraDataVec,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let mut shortest_path_tree = shortests_path_tree(data);
    let mut shortcuts = Vec::new();

    let mut stack = vec![vertex as usize];

    let mut label = Label::new(vertex);
    while let Some(tail) = stack.pop() {
        let mut heads = std::mem::take(&mut shortest_path_tree[tail]);

        while let Some(head) = heads.pop() {
            if order[head as usize] > order[tail] {
                stack.push(head as usize);
                label.entries.push(LabelEntry {
                    vertex: head,
                    predecessor: Some(tail as VertexId),
                    weight: data.vertices[head as usize].weight.unwrap(),
                });
            } else {
                for &tail_child in std::mem::take(&mut shortest_path_tree[head as usize]).iter() {
                    heads.push(tail_child);
                    let edge = DirectedEdge::new(tail as VertexId, tail_child).unwrap();
                    shortcuts.push((edge, tail_child));
                }
            }
        }
    }

    label.entries.sort_unstable_by_key(|entry| entry.vertex);

    (label, shortcuts)
}
