use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Graph, Vertex, WeightedEdge,
    },
    search::{
        ch::{contracted_graph::ContractedGraph, contraction::contraction_with_witness_search},
        dijkstra::dijkstra_one_to_one_wrapped,
        hl::{
            brute_force::brute_force,
            hub_graph::{self, overlapp, HubLabelEntry},
        },
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::{thread_rng, Rng};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("read_edges_from_fmi_file");
    let edges = read_edges_from_fmi_file(&args.graph);

    println!("build graph");
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!("cloning out graph");
    let cloned_graph = graph.clone();

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_witness_search(graph);
    println!("create graph");
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    // let other_contracted_graph =
    //     brute_force_contracted_graph(&cloned_graph,
    // &contracted_graph.level_to_vertex);

    println!("brute_force");
    let hub_graph = brute_force(&cloned_graph, &contracted_graph.vertex_to_level);

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());

            let forward_label = hub_graph.forward.get_label(source);
            let backward_label = hub_graph.backward.get_label(target);

            let start = Instant::now();
            let hl_distance = overlapp(forward_label, backward_label).map(|(distance, _)| distance); //ch_one_to_one_wrapped(&other_contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let start = Instant::now();
            let dijkstra_distance =
                dijkstra_one_to_one_wrapped(cloned_graph.out_graph(), source, target)
                    .map(|path| path.distance);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&hl_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect::<Vec<_>>();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}

fn create_half_hub_graph(graph: &dyn Graph, level_to_vertex: &Vec<Vertex>) {
    let mut labels = (0..graph.number_of_vertices())
        .map(|vertex| vec![HubLabelEntry::new(vertex)])
        .collect_vec();

    for &vertex in level_to_vertex.iter() {
        let mut neighbor_labels = graph
            .edges(vertex)
            .map(|edge| labels.get(edge.head as usize).unwrap())
            .collect::<Vec<_>>();
        neighbor_labels.push(labels.get(vertex as usize).unwrap());

        labels[vertex as usize] = merge_labels(&neighbor_labels);
    }
}

fn merge_labels(labels: &Vec<&Vec<HubLabelEntry>>) -> Vec<HubLabelEntry> {
    let mut new_label = Vec::new();

    let mut labels = labels
        .iter()
        .map(|label| label.iter().peekable())
        .collect_vec();

    while !labels.is_empty() {
        let mut min_entry = HubLabelEntry {
            vertex: Vertex::MAX,
            distance: Distance::MAX,
            predecessor_index: None,
        };

        let mut labels_with_min_vertex = Vec::new();

        for label in labels.iter_mut() {
            let entry = *label.peek().unwrap();

            if entry.vertex < min_entry.vertex {
                min_entry = entry.clone();
                labels_with_min_vertex.clear();
                labels_with_min_vertex.push(label);
            } else if entry.vertex == min_entry.vertex {
                labels_with_min_vertex.push(label);
            }
        }

        new_label.push(min_entry);

        for label in labels_with_min_vertex.iter_mut() {
            label.next();
        }

        // Retain only non-empty iterators
        labels.retain_mut(|label| label.peek().is_some());
    }

    new_label
}
