use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    process::exit,
    time::Instant,
    usize,
};

use ahash::{HashSet, HashSetExt};
use clap::Parser;
use faster_paths::{
    ch::{all_in_preprocessor::AllInPrerocessor, preprocessor::Preprocessor},
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
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

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

    let start = Instant::now();
    let labels: Vec<_> = test_cases
        .par_iter()
        .take(1_000)
        .progress()
        .map(|test_case| {
            let forward_label = get_out_label(test_case.request.source(), &graph, &order);
            let backward_label = get_out_label(test_case.request.target(), &graph, &order);

            let mut weight = None;
            if let Some((this_weight, _, _)) = HubGraph::overlap(&forward_label, &backward_label) {
                weight = Some(this_weight);
            }

            if weight != test_case.weight {
                println!("err soll {:?}, ist {:?}", test_case.weight, weight);
            }
            forward_label
        })
        .collect();
    println!("all {} tests passed", test_cases.len());

    println!(
        "average label size is {}",
        labels
            .iter()
            .map(|label| label.entries.len())
            .sum::<usize>()
            / labels.len()
    );

    println!(
        "took {:?} for {} labels which means {:?} for whole graph (half if bidirectional)",
        start.elapsed(),
        labels.len(),
        start.elapsed() / labels.len() as u32 * graph.number_of_vertices()
    );
}

fn get_out_label(vertex: VertexId, graph: &dyn Graph, order: &[u32]) -> Label {
    let dijkstra = Dijkstra::new(graph);
    let data = dijkstra.single_source(vertex);

    let mut children = vec![Vec::new(); graph.number_of_vertices() as usize];

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

fn get_in_label(vertex: VertexId, graph: &dyn Graph, order: &[u32]) -> Label {
    let dijkstra = Dijkstra::new(graph);
    let data = dijkstra.single_target(vertex);

    let mut children = vec![Vec::new(); graph.number_of_vertices() as usize];

    for (vertex, entry) in data.vertices.iter().enumerate() {
        if let Some(predecessor) = entry.predecessor {
            children[predecessor as usize].push(vertex);
        }
    }

    let mut stack = vec![vertex as usize];

    let mut label = Label::new(vertex);
    while let Some(current) = stack.pop() {
        let mut current_children = std::mem::take(&mut children[current as usize]);

        while let Some(child) = current_children.pop() {
            let child_children = std::mem::take(&mut children[child]);

            if order[child] > order[current] {
                stack.extend(child_children);
                label.entries.push(LabelEntry {
                    vertex: child as VertexId,
                    predecessor: Some(current as VertexId),
                    weight: data.vertices[child].weight.unwrap(),
                });
            } else {
                current_children.extend(child_children);
            }
        }
    }

    label.entries.sort_unstable_by_key(|entry| entry.vertex);

    label
}

fn get_small_graph() -> ReversibleVecGraph {
    // https://jlazarsfeld.github.io/ch.150.project/img/contraction/contract-full-1.png
    let mut graph = ReversibleVecGraph::new();
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(0, 1, 3).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(0, 2, 5).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(0, 10, 3).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(1, 2, 3).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(1, 3, 5).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(2, 3, 2).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(2, 9, 2).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(3, 4, 7).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(3, 9, 4).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(4, 5, 6).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(4, 9, 3).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(5, 6, 4).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(5, 7, 2).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(6, 7, 3).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(6, 8, 5).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(7, 8, 3).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(7, 9, 2).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(8, 9, 4).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(8, 10, 6).unwrap());
    add_edge_bidrectional(&mut graph, &DirectedWeightedEdge::new(9, 10, 3).unwrap());
    graph
}
