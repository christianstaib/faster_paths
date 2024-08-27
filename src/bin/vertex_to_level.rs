use std::{collections::HashSet, fs::File, io::BufWriter, path::PathBuf};

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
        dijkstra::dijktra_one_to_all,
    },
    utility::get_progressbar_long_jobs,
};
use indicatif::ParallelProgressIterator;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

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

    let edges = read_edges_from_fmi_file(&args.graph);

    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let paths = get_paths(
        graph.out_graph(),
        args.number_of_searches,
        args.number_of_paths_per_search,
    );

    let level_to_vertex = level_to_vertex(&paths, graph.out_graph().number_of_vertices());
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let writer = BufWriter::new(File::create(args.vertex_to_level).unwrap());
    serde_json::to_writer(writer, &vertex_to_level).unwrap();
}

/// Computes paths in a graph using Dijkstra's algorithm.
pub fn get_paths(
    graph: &dyn Graph,
    number_of_searches: u32,
    number_of_paths_per_search: u32,
) -> Vec<Vec<Vertex>> {
    (0..number_of_searches)
        .into_par_iter()
        .progress_with(get_progressbar_long_jobs(
            "Getting many paths",
            number_of_searches as u64,
        ))
        .map_init(
            || {
                (
                    // Reuse data structures.
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

                // Clear the data structures for reuse in the next iteration
                data.clear();
                expanded.clear();
                queue.clear();

                paths
            },
        )
        .flatten()
        .collect()
}

/// Constructs a vector of vertices ordered by their level, where each level is
/// determined by the vertex that hits the most paths. The function processes
/// paths in parallel, iteratively selecting the most frequent vertex and
/// updating active paths until all paths are exhausted.
pub fn level_to_vertex(paths: &[Vec<Vertex>], number_of_vertices: u32) -> Vec<Vertex> {
    let mut level_to_vertex = Vec::new();
    let mut active_paths: Vec<usize> = (0..paths.len()).collect();
    let mut active_vertices: HashSet<Vertex> = HashSet::from_iter(0..number_of_vertices);

    let pb = get_progressbar_long_jobs("Generating level_to_vertex vector", paths.len() as u64);
    while !active_paths.is_empty() {
        let hits = active_paths
            // Split the active_paths into chunks for parallel processing.
            .par_chunks(active_paths.len().div_ceil(rayon::current_num_threads()))
            // For each chunk, calculate how frequently each vertex appears across the active paths.
            .map(|indices| {
                let mut partial_hits = vec![0; number_of_vertices as usize];
                for &index in indices {
                    for &vertex in paths[index].iter() {
                        partial_hits[vertex as usize] += 1;
                    }
                }
                partial_hits
            })
            // Sum the results from all threads to get the total hit count for each vertex.
            .reduce(
                || vec![0; number_of_vertices as usize],
                |mut hits, partial_hits| {
                    for index in 0..number_of_vertices as usize {
                        hits[index] += partial_hits[index]
                    }
                    hits
                },
            );

        // Get the vertex that hits the most paths.
        let vertex = hits
            .iter()
            .enumerate()
            .max_by_key(|&(_vertex, hits)| hits)
            .map(|(vertex, _)| vertex as Vertex)
            .expect("hits cannot be empty if number_of_vertices > 0");

        // There is no real max hitting vertex anymore.
        if hits[vertex as usize] == 1 {
            break;
        }

        // The level of this vertex is 1 lower than the previous one.
        level_to_vertex.insert(0, vertex);

        active_vertices.remove(&(vertex));

        // Remove paths that are hit by vertex.
        active_paths = active_paths
            .into_par_iter()
            .filter(|&index| !paths[index].contains(&vertex))
            .collect();

        pb.set_position((paths.len() - active_paths.len()) as u64);
    }
    pb.finish_and_clear();

    // Insert the remaining vertices at the front, e.g. assign them the lowest
    // levels.
    level_to_vertex.splice(0..0, active_vertices);

    level_to_vertex
}
