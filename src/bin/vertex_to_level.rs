use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    fs::File,
    io::BufWriter,
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, Vertex,
    },
    search::{
        ch::contracted_graph::vertex_to_level,
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
        },
        dijkstra::{dijkstra_one_to_one_wrapped, dijktra_one_to_all},
        hl::hub_graph::{self, get_path_from_overlapp, HubGraph},
    },
};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    #[arg(short, long)]
    number_of_searches: u32,
    #[arg(short, long)]
    number_of_paths_per_search: u32,
    #[arg(short, long)]
    vertex_to_level: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("read_edges_from_fmi_file");
    let edges = read_edges_from_fmi_file(&args.graph);

    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let paths = get_paths(
        graph.out_graph(),
        args.number_of_searches,
        args.number_of_paths_per_search,
    );

    println!("getting levels");
    let level_to_vertex = level_to_vertex(&paths, graph.out_graph().number_of_vertices());
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    println!("writing levels");
    let writer = BufWriter::new(File::create(args.vertex_to_level).unwrap());
    serde_json::to_writer(writer, &vertex_to_level).unwrap();

    let hub_graph = HubGraph::by_brute_force(&graph, &vertex_to_level);
    println!("average label size is {}", hub_graph.average_label_size());

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..graph.out_graph().number_of_vertices());

            let start = Instant::now();
            let hl_path = get_path_from_overlapp(
                hub_graph.forward.get_label(source),
                hub_graph.backward.get_label(target),
                &hub_graph.shortcuts,
            );
            // let mut hl_path = ch_one_to_one_wrapped(&contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let hl_distance = hl_path.as_ref().map(|path| path.distance);

            let distance =
                hl_path.and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
            assert_eq!(distance, hl_distance);

            let start = Instant::now();
            let dijkstra_distance = dijkstra_one_to_one_wrapped(graph.out_graph(), source, target)
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

pub fn get_paths(
    graph: &dyn Graph,
    number_of_searches: u32,
    number_of_paths_per_search: u32,
) -> Vec<Vec<Vertex>> {
    (0..number_of_searches)
        .into_par_iter()
        .progress()
        .map_init(
            || {
                (
                    DijkstraDataVec::new(graph),
                    VertexExpandedDataBitSet::new(graph),
                    VertexDistanceQueueBinaryHeap::new(),
                    thread_rng(),
                )
            },
            |(data, expanded, queue, rng), _| {
                let mut paths = Vec::new();
                let source = rng.gen_range(0..graph.number_of_vertices());
                dijktra_one_to_all(graph, data, expanded, queue, source);

                for _ in 0..number_of_paths_per_search {
                    let target = rng.gen_range(0..graph.number_of_vertices());
                    if let Some(path) = data.get_path(target) {
                        paths.push(path.vertices);
                    }
                }

                data.clear();
                expanded.clear();
                queue.clear();

                paths
            },
        )
        .flatten()
        .collect()
}

pub fn level_to_vertex(paths: &[Vec<Vertex>], number_of_vertices: u32) -> Vec<Vertex> {
    let mut level_to_vertex = Vec::new();

    let mut active_paths: Vec<usize> = (0..paths.len()).collect();
    let mut active_vertices: HashSet<Vertex> = HashSet::from_iter(0..number_of_vertices);

    let mut all_hits = (0..number_of_vertices).map(|_| 0).collect_vec();

    let pb = ProgressBar::new(active_paths.len() as u64);
    while !active_paths.is_empty() {
        let number_of_hits: HashMap<Vertex, u32> = active_paths
            .par_iter()
            .map(|&index| &paths[index])
            .map(|path| HashMap::from_iter(path.iter().map(|&vertex| (vertex, 1))))
            .reduce(
                || HashMap::new(),
                |mut acc, local_hits| {
                    for (&vertex, &hits) in local_hits.iter() {
                        acc.entry(vertex).and_modify(|v| *v += hits).or_insert(hits);
                    }
                    acc
                },
            );

        let (&max_hitting_vertex, &max_hits) = number_of_hits
            .iter()
            .max_by_key(|&(_vertex, hits)| hits)
            .unwrap();

        if max_hits == 1 {
            println!("early exit");
            break;
        }

        number_of_hits
            .iter()
            .for_each(|(&vertex, &hits)| all_hits[vertex as usize] += hits);

        level_to_vertex.push(max_hitting_vertex);
        active_vertices.remove(&max_hitting_vertex);

        active_paths = active_paths
            .into_par_iter()
            .filter(|&paths_idx| !paths[paths_idx].contains(&max_hitting_vertex))
            .collect();

        pb.set_position((paths.len() - active_paths.len()) as u64);
    }
    pb.finish_and_clear();

    let mut active_vertices = active_vertices.into_iter().collect_vec();
    active_vertices.sort_unstable_by_key(|&vertex| Reverse(all_hits[vertex as usize]));
    level_to_vertex.extend(active_vertices);

    level_to_vertex.reverse();

    level_to_vertex
}
