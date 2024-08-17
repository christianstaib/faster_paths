use std::{option, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Edge, Graph, Vertex, WeightedEdge,
    },
    search::{
        ch::{
            contracted_graph::{self, ContractedGraph},
            contraction::contraction_with_witness_search,
        },
        dijkstra::dijkstra_one_to_one_wraped,
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

    println!("Reading edges");
    let mut edges = read_edges_from_fmi_file(&args.graph);
    let out_graph = VecVecGraph::from_edges(&edges);

    println!("Building graph");
    let mut graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let (level_to_vertex, shortcuts) = contraction_with_witness_search(graph);

    shortcuts.iter().for_each(|(&(tail, head), &weight)| {
        let edge = WeightedEdge { tail, head, weight };
        edges.push(edge);
    });

    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let (up_edges, down_edges): (Vec<WeightedEdge>, Vec<WeightedEdge>) =
        edges.into_iter().partition(|edge| {
            vertex_to_level[edge.tail as usize] < vertex_to_level[edge.head as usize]
        });

    println!(
        "there are {} up edges and {} down edges",
        up_edges.len(),
        down_edges.len()
    );

    let up_graph = VecVecGraph::from_edges(&up_edges);
    let down_graph =
        VecVecGraph::from_edges(&down_edges.iter().map(|edge| edge.reversed()).collect_vec());

    let contracted_graph = ContractedGraph {
        up_graph,
        down_graph,
        level_to_vertex,
    };

    for _ in 0..10 {
        let source = thread_rng().gen_range(0..contracted_graph.up_graph.number_of_vertices());
        let target = thread_rng().gen_range(0..contracted_graph.up_graph.number_of_vertices());

        let ch_distance = contracted_graph.shortest_path_distance(source, target);
        let dijkstra_distance = dijkstra_one_to_one_wraped(&out_graph, source, target);

        println!("{:?} {:?}", ch_distance, dijkstra_distance);
    }
}

pub fn vertex_to_level(level_to_vertex: &Vec<Vertex>) -> Vec<u32> {
    let mut vertex_to_level = vec![0; level_to_vertex.len()];

    for (level, &vertex) in level_to_vertex.iter().enumerate() {
        vertex_to_level[vertex as usize] = level as u32;
    }

    vertex_to_level
}

// println!("Reading test cases");
// let mut reader = BufReader::new(File::open(&args.tests).unwrap());
// let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(&mut
// reader).unwrap();
// let graph = graph.out_graph();
// let graph = &graph;
// for _ in (0..200).progress().progress() {
//     let source = thread_rng().gen_range(0..graph.number_of_vertices());
//     let target = thread_rng().gen_range(0..graph.number_of_vertices());

//     let mut data = DijkstraDataVec::new(graph);
//     let mut expanded = VertexExpandedDataBitSet::new(graph);
//     let mut queue = VertexDistanceQueueDaryHeap::<3>::new();

//     let start = Instant::now();
//     dijktra_one_to_one(graph, &mut data, &mut expanded, &mut queue,
// source, target);     duration += start.elapsed();
// }
// println!(
//     "average duration was {:?}",
//     duration / (test_cases.len() as u32)
// );
