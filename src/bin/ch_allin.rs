use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
        WeightedEdge,
    },
    search::{
        alt::landmark::Landmarks, ch::contracted_graph::ContractedGraph, DistanceHeuristic,
        PathFinding,
    },
    utility::get_progressbar,
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

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
    let mut graph_org: ReversibleGraph<VecVecGraph> = bincode::deserialize_from(reader).unwrap();

    if !graph_org.out_graph().is_bidirectional() {
        let edges = graph_org.out_graph().all_edges();
        for edge in edges.iter() {
            graph_org.set_weight(&edge.remove_weight(), Some(edge.weight));
            graph_org.set_weight(&edge.remove_weight().reversed(), Some(edge.weight));
        }
    }
    let mut edges: HashMap<(Vertex, Vertex), Distance> = graph_org
        .out_graph()
        .all_edges()
        .iter()
        .map(|edge| ((edge.tail, edge.head), edge.weight))
        .collect();

    println!(
        "graph is bidirectional? {}",
        graph_org.out_graph().is_bidirectional()
    );

    let landmarks = Landmarks::random(&graph_org, 0 * rayon::current_num_threads() as u32);

    let shortcuts = HashMap::new();

    let mut graph = ArrayGraph::new(graph_org.out_graph());

    let mut diffs = (0..graph.num_vertices)
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let diff = edge_diff(&graph, &landmarks, vertex as Vertex);
            Reverse((diff, vertex as Vertex))
        })
        .collect::<BinaryHeap<_>>();

    println!(
        "min diff {}",
        diffs
            .iter()
            .map(|Reverse((diff, _vertex))| diff)
            .min()
            .unwrap()
    );

    let mut writer = BufWriter::new(File::create("all_in.txt").unwrap());

    let mut level_to_vertex = Vec::new();
    let pb = get_progressbar("contracting", diffs.len() as u64);
    while let Some(Reverse((old_diff, vertex))) = diffs.pop() {
        let num_edges = graph
            .array
            .iter()
            .filter(|&&distance| distance != Distance::MAX)
            .count();
        let num_vertices = diffs.len() + 1;
        // let new_diff = edge_diff(&graph, graph_org.out_graph(), vertex);
        // if new_diff > old_diff {
        //     diffs.push(Reverse((new_diff, vertex)));
        //     continue;
        // }
        pb.inc(1);
        level_to_vertex.push(vertex);

        let start = Instant::now();

        let this_edges = contract(&mut graph, &landmarks, vertex)
            .into_par_iter()
            .filter(|edge| {
                edge.weight < *edges.get(&(edge.tail, edge.head)).unwrap_or(&Distance::MAX)
            })
            .collect::<Vec<_>>();

        this_edges.into_iter().for_each(|edge| {
            edges.insert((edge.tail, edge.head), edge.weight);
        });

        let iteration_duration = start.elapsed();
        writeln!(
            writer,
            "{} {} {}",
            num_vertices,
            num_edges,
            start.elapsed().as_secs_f32()
        )
        .unwrap();
    }
    writer.flush().unwrap();

    let edges = edges
        .into_par_iter()
        .map(|((tail, head), weight)| WeightedEdge::new(tail, head, weight))
        .collect();

    let ch = ContractedGraph::new(level_to_vertex, edges, shortcuts);

    println!("upward edges {}", ch.upward_graph().number_of_edges());
    println!("downward edges {}", ch.downward_graph().number_of_edges());

    let n = 10_000;
    let mut time = Duration::new(0, 0);
    for _ in 0..n {
        let (&source, &target) = graph_org
            .out_graph()
            .vertices()
            .collect_vec()
            .choose_multiple(&mut thread_rng(), 2)
            .collect_tuple()
            .unwrap();

        let start = Instant::now();
        let ch_dist = ch.shortest_path_distance(source, target);
        time += start.elapsed();

        assert_eq!(
            graph_org.out_graph().shortest_path_distance(source, target),
            ch_dist
        );
    }
    println!("all ok. average time was {:?}", time / n);
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
    pub fn new(graph: &dyn Graph) -> Self {
        let edges = graph.all_edges();

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
            if edge.weight < array_graph.get_weight(edge.tail, edge.head) {
                array_graph.set_weight(edge.tail, edge.head, edge.weight);
            }
            assert_eq!(
                array_graph.get_weight(edge.tail, edge.head),
                graph
                    .get_weight(&edge.remove_weight())
                    .unwrap_or(Distance::MAX)
            );

            if edge.tail == edge.head {
                println!("{:?}", edge);
            }
        }

        array_graph
    }

    pub fn get_weight(&self, tail: Vertex, head: Vertex) -> Distance {
        if tail == head {
            return 0;
        }
        self.array[get_index(tail, head)]
    }

    pub fn set_weight(&mut self, tail: Vertex, head: Vertex, weight: Distance) {
        if tail == head {
            return;
        }
        self.array[get_index(tail, head)] = weight
    }
}

fn contract(
    graph: &mut ArrayGraph,
    heuristic: &dyn DistanceHeuristic,
    vertex: Vertex,
) -> Vec<WeightedEdge> {
    let neighbors_and_edge_weight = (0..graph.num_vertices)
        .into_par_iter()
        .filter(|&head| head as Vertex != vertex)
        .map(|head| (head as Vertex, graph.get_weight(vertex, head as Vertex)))
        .filter(|&(_vertex, edge_weight)| edge_weight != Distance::MAX)
        .collect::<Vec<_>>();

    let edges = neighbors_and_edge_weight
        .par_iter()
        .map(|&(tail, tail_weight)| {
            let mut sub_edges = Vec::new();
            let mut to_update = Vec::new();

            for &(head, head_weight) in neighbors_and_edge_weight.iter() {
                if tail == head {
                    continue;
                }

                let alternative_weight = tail_weight + head_weight;
                if alternative_weight < graph.get_weight(tail, head) {
                    if heuristic.is_less_or_equal_upper_bound(tail, head, alternative_weight) {
                        to_update.push((tail, head, alternative_weight));
                        // graph.set_weight(tail, head, alternative_weight);
                        sub_edges.push(WeightedEdge::new(tail, head, alternative_weight));
                        sub_edges.push(WeightedEdge::new(head, tail, alternative_weight));
                    }
                }
            }

            (to_update, sub_edges)
        })
        // .flatten()
        .collect::<Vec<_>>();

    let edges = edges
        .into_iter()
        .map(|(to_update, sub_edges)| {
            for (tail, head, weight) in to_update {
                graph.set_weight(tail, head, weight);
            }
            sub_edges
        })
        .flatten()
        .collect();

    neighbors_and_edge_weight
        .iter()
        .for_each(|&(head, _)| graph.set_weight(vertex, head, Distance::MAX));

    edges
}

fn edge_diff(graph: &ArrayGraph, heuristic: &dyn DistanceHeuristic, vertex: Vertex) -> i64 {
    let neighbors_and_edge_weight = (0..graph.num_vertices)
        .into_par_iter()
        .filter(|&head| head as Vertex != vertex)
        .map(|head| (head as Vertex, graph.get_weight(vertex, head as Vertex)))
        .filter(|&(_vertex, edge_weight)| edge_weight != Distance::MAX)
        .collect::<Vec<_>>();

    //  assert_eq!(
    //      neighbors_and_edge_weight.len(),
    //      test_graph.neighbors(vertex).len()
    //  );
    // println!("num neighbors {}", neighbors_and_edge_weight.len());

    let new_edges: i64 = neighbors_and_edge_weight
        .par_iter()
        .map(|&(tail, _tail_weight)| {
            let mut new_edges = 0;
            for &(head, _head_weight) in neighbors_and_edge_weight.iter() {
                if tail == head {
                    continue;
                }

                let distance = graph.get_weight(tail, head);
                // assert_eq!(
                //     test_graph
                //         .get_weight(&Edge { tail, head })
                //         .unwrap_or(Distance::MAX),
                //     distance
                // );

                if distance == Distance::MAX
                    && heuristic.is_less_or_equal_upper_bound(
                        tail,
                        head,
                        _tail_weight + _head_weight,
                    )
                {
                    new_edges += 1;
                }
            }
            new_edges
        })
        .sum();
    // println!("num new edges {}", new_edges);

    new_edges as i64 - neighbors_and_edge_weight.len() as i64
}
