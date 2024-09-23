use std::{
    fs::File,
    io::BufWriter,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Graph, Vertex,
    },
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataHashSet},
        },
        dijkstra::{dijkstra_one_to_all_wraped, dijkstra_one_to_one},
    },
    utility::{benchmark_distances, benchmark_path, gen_tests_cases, get_progressbar},
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    degrees_out: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!(
        "Is graph bidirectional? {}",
        graph.out_graph().is_bidirectional()
    );

    if let Some(degrees_out) = args.degrees_out.as_ref() {
        let out_degrees = graph
            .out_graph()
            .vertices()
            .map(|vertex| graph.out_graph().edges(vertex).len())
            .collect_vec();
        let writer = BufWriter::new(File::create(&degrees_out).unwrap());
        serde_json::to_writer(writer, &out_degrees).unwrap();
    }

    let vertices = graph.out_graph().vertices().collect_vec();
    println!(
        "non trivial vertices: {}",
        graph.out_graph().non_trivial_vertices().len()
    );
    println!("average degree is {}", graph.out_graph().average_degree());
    println!(
        "sum of squared degree {}",
        graph
            .out_graph()
            .vertices()
            .map(|vertex| graph.out_graph().edges(vertex).len().pow(2) as u128)
            .sum::<u128>()
    );

    let n = 10_000;
    let (avg_path_len, avg_dijkstra_rank, avg_queue_pops) = get_dijkstra_info(&graph, n);

    println!("Values over {} parallel searches", n);
    println!("average path hops len {}", avg_path_len);
    println!("average dijkstra_rank {}", avg_dijkstra_rank);
    println!("average queue pops {}", avg_queue_pops);

    let start = Instant::now();
    (0..n).into_par_iter().for_each_init(
        || thread_rng(),
        |mut rng, _| {
            let source = vertices.choose(&mut rng).cloned().unwrap();

            dijkstra_one_to_all_wraped(graph.out_graph(), source);
        },
    );
    let pred_time = start.elapsed().as_secs_f64() * 2.0 / n as f64 * vertices.len() as f64;
    println!(
        "2xDijsktra per node for all nodes would take {:?}",
        Duration::from_secs_f64(pred_time)
    );

    let m = 1_00;
    println!("Value over {} sequential searches", m);
    let sources_and_targets = gen_tests_cases(graph.out_graph(), m);
    let avg_dijkstra_duration = benchmark_path(graph.out_graph(), &sources_and_targets);
    println!(
        "Average dijkstra duration for path creation is {:?}",
        avg_dijkstra_duration
    );

    let avg_dijkstra_duration = benchmark_distances(graph.out_graph(), &sources_and_targets);
    println!(
        "Average dijkstra duration for path distance is {:?}",
        avg_dijkstra_duration
    );
}

fn get_dijkstra_info(graph: &ReversibleGraph<VecVecGraph>, n: u64) -> (f32, f32, f32) {
    let non_trivial_vertices = graph.out_graph().non_trivial_vertices();
    let pb = get_progressbar("Gettings paths", n);
    let data = (0..)
        .par_bridge()
        .map_init(
            || {
                let data = DijkstraDataVec::new(graph.out_graph());
                let queue = QueueWrapper {
                    number_of_pops: 0,
                    queue: Box::new(VertexDistanceQueueBinaryHeap::new()),
                };
                let expanded = VertexExpandedDataHashSet::new();
                let rng = thread_rng();

                (data, queue, expanded, rng)
            },
            |(data, queue, expanded, rng), _| {
                let source_and_target = non_trivial_vertices.choose_multiple(rng, 2).collect_vec();
                dijkstra_one_to_one(
                    graph.out_graph(),
                    data,
                    expanded,
                    queue,
                    *source_and_target[0],
                    *source_and_target[1],
                );
                let path = data.get_path(*source_and_target[1]);

                let dijkstra_rank = expanded.dijkstra_rank();
                let queue_pops = queue.number_of_pops;

                data.clear();
                queue.clear();
                expanded.clear();

                (path, dijkstra_rank, queue_pops)
            },
        )
        .filter_map(|(path, dijkstra_rank, queue_pops)| {
            Some((path?.vertices.len(), dijkstra_rank, queue_pops))
        })
        .take_any(n as usize)
        .progress_with(pb)
        .collect::<Vec<_>>();

    let avg_path_len =
        data.iter().map(|&(len, _, _)| len as u64).sum::<u64>() as f32 / data.len() as f32;
    let avg_dijkstra_rank =
        data.iter().map(|&(_, rank, _)| rank as u64).sum::<u64>() as f32 / data.len() as f32;
    let avg_queue_pops =
        data.iter().map(|&(_, _, pops)| pops as u64).sum::<u64>() as f32 / data.len() as f32;
    (avg_path_len, avg_dijkstra_rank, avg_queue_pops)
}

pub struct QueueWrapper {
    pub number_of_pops: u32,
    pub queue: Box<dyn VertexDistanceQueue>,
}

impl VertexDistanceQueue for QueueWrapper {
    fn clear(&mut self) {
        self.number_of_pops = 0;
        self.queue.clear()
    }

    fn insert(&mut self, vertex: Vertex, distance: Distance) {
        self.queue.insert(vertex, distance)
    }

    fn pop(&mut self) -> Option<(Vertex, Distance)> {
        self.number_of_pops += 1;
        self.queue.pop()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn peek(&mut self) -> Option<(Vertex, Distance)> {
        self.queue.peek()
    }
}
