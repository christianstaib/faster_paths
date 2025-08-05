use std::path::{Path, PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Graph, Vertex,
    },
    search::{
        ch::{brute_force::get_ch_edges_debug, contracted_graph::vertex_to_level},
        collections::{
            dijkstra_data::DijkstraDataVec, vertex_distance_queue::VertexDistanceQueueBinaryHeap,
            vertex_expanded_data::VertexExpandedDataBitSet,
        },
        PathFinding,
    },
    utility::{read_json_with_spinnner, write_json_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    level_to_vertex: PathBuf,
}

fn main() {
    let args = Args::parse();

    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    // Read vertex_to_level
    let level_to_vertex: Vec<Vertex> =
        read_json_with_spinnner("level to vertex", args.level_to_vertex.as_path());
    assert!(level_to_vertex
        .iter()
        .all(|&v| v < graph.number_of_vertices()));
    let vertex_to_level = vertex_to_level(&level_to_vertex);
    assert!(level_to_vertex
        .iter()
        .all(|&v| v < graph.number_of_vertices()));

    let visited = graph
        .out_graph()
        .vertices()
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let mut data = DijkstraDataVec::new(graph.out_graph());
            let (_edges, _shortcutes, alive_setteled, setteled, seen) = get_ch_edges_debug(
                graph.out_graph(),
                &mut data,
                &mut VertexExpandedDataBitSet::new(graph.out_graph()),
                &mut VertexDistanceQueueBinaryHeap::new(),
                &vertex_to_level,
                vertex,
            );

            data.distances
                .into_iter()
                .filter(|&d| d != Distance::MAX)
                .count()
        })
        .collect::<Vec<_>>();

    write_json_with_spinnner("visited", Path::new("visited.json"), &visited);

    // println!("{:?}", alive_setteled);
    // println!("{:?}", setteled);
    // println!("{:?}", seen);
    // println!("{:?}", _edges.iter().map(|edge| edge.head).collect_vec());
}
