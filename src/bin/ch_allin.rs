use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
        WeightedEdge,
    },
    search::{DistanceHeuristic, PathFinding},
    utility::get_progressbar,
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

pub struct PathfinderHeuristic<'a> {
    pub pathfinder: &'a dyn PathFinding,
}

impl<'a> DistanceHeuristic for PathfinderHeuristic<'a> {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Distance {
        self.pathfinder
            .shortest_path_distance(source, target)
            .unwrap_or(0)
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Distance {
        self.pathfinder
            .shortest_path_distance(source, target)
            .unwrap_or(Distance::MAX)
    }
}

fn main() {
    let args = Args::parse();

    // Build graph
    let reader = BufReader::new(File::open(&args.graph).unwrap());
    let graph: ReversibleGraph<VecVecGraph> = bincode::deserialize_from(reader).unwrap();

    let mut graph = ArrayGraph::new(&graph.out_graph().all_edges());

    for vertex in (0..graph.num_vertices)
    // .progress_with(get_progressbar("set up queue", graph.num_vertices as u64))
    {
        let diff = edge_diff(&mut graph, vertex as Vertex);
        println!("{}", diff);
    }
}

fn get_index(tail: Vertex, head: Vertex) -> usize {
    let min = std::cmp::min(tail, head) as usize;
    let max = std::cmp::max(tail, head) as usize;
    (((max - 1) * max) / 2) + min
}

pub struct ArrayGraph {
    pub num_vertices: usize,
    pub array: Vec<Vertex>,
}

impl ArrayGraph {
    pub fn new(edges: &Vec<WeightedEdge>) -> Self {
        let max_vertex = edges.iter().map(|edge| edge.tail).max().unwrap();
        println!("edges len {}", edges.len());
        println!("max vertex is {}", max_vertex);
        println!("aray len is {}", get_index(max_vertex + 1, 0));
        let array = vec![Distance::MAX; get_index(max_vertex + 1, 0)];

        let mut array_graph = ArrayGraph {
            num_vertices: max_vertex as usize + 1,
            array,
        };

        for edge in edges.iter().progress() {
            if edge.tail < edge.head {
                if edge.weight < array_graph.get_weight(edge.tail, edge.head) {
                    array_graph.set_weight(edge.tail, edge.head, edge.weight);
                }
            }

            if edge.tail == edge.head {
                println!("{:?}", edge);
            }
        }

        array_graph
    }

    pub fn get_weight(&self, tail: Vertex, head: Vertex) -> Distance {
        assert!(tail != head);
        self.array[get_index(tail, head)]
    }

    pub fn set_weight(&mut self, tail: Vertex, head: Vertex, weight: Distance) {
        assert!(tail != head);
        self.array[get_index(tail, head)] = weight
    }
}

fn contract(graph: &mut ArrayGraph, vertex: Vertex) {
    let neighbors_and_edge_weight = (0..graph.num_vertices)
        .into_par_iter()
        .map(|head| (head as Vertex, graph.get_weight(vertex, head as Vertex)))
        .filter(|&(_vertex, edge_weight)| edge_weight != Distance::MAX)
        .collect::<Vec<_>>();

    neighbors_and_edge_weight
        .iter()
        .for_each(|&(tail, tail_weight)| {
            neighbors_and_edge_weight
                .iter()
                .for_each(|&(head, head_weight)| {
                    if tail < head {
                        if tail_weight + head_weight < graph.get_weight(tail, head) {
                            graph.set_weight(tail, head, tail_weight + head_weight);
                        }
                    }
                })
        });

    neighbors_and_edge_weight
        .iter()
        .for_each(|&(head, _)| graph.set_weight(vertex, head, Distance::MAX));
}

fn edge_diff(graph: &mut ArrayGraph, vertex: Vertex) -> i32 {
    let neighbors_and_edge_weight = (0..graph.num_vertices)
        .into_par_iter()
        .filter(|&head| vertex < head as u32)
        .map(|head| (head as Vertex, graph.get_weight(vertex, head as Vertex)))
        .filter(|&(_vertex, edge_weight)| edge_weight != Distance::MAX)
        .collect::<Vec<_>>();

    let mut new_edges = 0;
    neighbors_and_edge_weight
        .iter()
        .for_each(|&(tail, _tail_weight)| {
            neighbors_and_edge_weight
                .iter()
                .for_each(|&(head, _head_weight)| {
                    if tail < head {
                        if graph.get_weight(tail, head) == Distance::MAX {
                            new_edges += 1;
                        }
                    }
                })
        });

    (2 * new_edges as i32) - (2 * neighbors_and_edge_weight.len() as i32)
}
