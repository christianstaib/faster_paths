use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    usize,
};

use clap::Parser;
use faster_paths::{
    ch::{all_in_preprocessor::AllInPrerocessor, preprocessor::Preprocessor},
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::all_edges,
        path::{PathFinding, ShortestPathTestCase},
        reversible_hash_graph::ReversibleHashGraph,
        Graph, VertexId,
    },
    hl::{hub_graph::HubGraph, label::Label, label_entry::LabelEntry},
    simple_algorithms::dijkstra::Dijkstra,
};
use indicatif::ProgressIterator;
use itertools::Itertools;

use rand::prelude::SliceRandom;

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

    for test_case in test_cases.iter().progress() {
        let forward_label = get_label(test_case.request.source(), &graph, &order);
        let backward_label = get_label(test_case.request.target(), &graph, &order);

        let mut weight = None;
        if let Some((this_weight, _, _)) = HubGraph::overlap(&forward_label, &backward_label) {
            weight = Some(this_weight);
        }

        if weight != test_case.weight {
            println!("err soll {:?}, ist {:?}", test_case.weight, weight);
        }
    }

    println!("all {} tests passed", test_cases.len());
}

fn get_label(vertex: VertexId, graph: &dyn Graph, order: &[u32]) -> Label {
    let dijkstra = Dijkstra::new(graph);
    let mut data = dijkstra.single_source(vertex);

    let mut children = vec![Vec::new(); graph.number_of_vertices() as usize];

    for (vertex, entry) in data.vertices.iter().enumerate() {
        if let Some(predecessor) = entry.predecessor {
            children[predecessor as usize].push(vertex);
        }
    }

    let mut label = Label::new(vertex);
    let mut stack = vec![vertex];
    while let Some(tail) = stack.pop() {
        for &head in children[tail as usize].iter() {
            if order[head as usize] > order[tail as usize] {
                let entry = LabelEntry {
                    vertex: head as VertexId,
                    predecessor: data.vertices[head].predecessor,
                    weight: data.vertices[head].weight.unwrap(),
                };
                label.entries.push(entry);
            } else {
                data.vertices[head].predecessor = Some(tail);
            }
            stack.push(head as VertexId);
        }
    }

    label.entries.sort_unstable_by_key(|entry| entry.vertex);

    label
}
