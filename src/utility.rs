use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use indicatif::*;
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    graphs::{Graph, Vertex},
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
        },
        dijkstra::{dijkstra_one_to_one_wrapped, dijktra_one_to_all},
        path::ShortestPathTestCase,
        PathFinding,
    },
};

pub fn get_progressbar_long_jobs(job_name: &str, len: u64) -> ProgressBar {
    let bar = ProgressBar::new(len);
    bar.set_message(job_name.to_string());
    bar.set_style(
        ProgressStyle::with_template(
            " {msg} {wide_bar} ({percent_precise}%) estimated remaining: {eta_precise}",
        )
        .unwrap(),
    );
    bar
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

pub fn benchmark_and_test(
    graph: &dyn Graph,
    tests: &[ShortestPathTestCase],
    pathfinder: &dyn PathFinding,
) -> Result<Duration, String> {
    // First only calculate paths. Checking correctness immediately would poison
    // cache.
    let path_and_duration = tests
        .into_iter()
        .progress_with(get_progressbar_long_jobs(
            "Benchmarking",
            tests.len() as u64,
        ))
        .map(|test| {
            let start = Instant::now();
            let path = pathfinder.shortest_path(test.source, test.target);
            (path, start.elapsed())
        })
        .collect_vec();

    for (test, (path, _duration)) in tests
        .iter()
        .zip(path_and_duration.iter())
        .progress_with(get_progressbar_long_jobs("Validating", tests.len() as u64))
    {
        // Test distance against test.
        let distance = path.as_ref().map(|path| path.distance);
        if distance != test.distance {
            return Err(format!(
                "Distance should be {:?} but is {:?} ({} -> {})",
                test.distance, distance, test.source, test.target
            )
            .to_string());
        }

        // Test path against test.
        let distance = path
            .as_ref()
            .and_then(|path| graph.get_path_distance(&path.vertices));
        if distance != test.distance {
            return Err(format!(
                "The path distance was correct but the path is wrong ({} -> {})",
                test.source, test.target
            )
            .to_string());
        }
    }

    let average_duration = path_and_duration
        .iter()
        .map(|(_path, duration)| duration)
        .sum::<Duration>()
        / path_and_duration.len() as u32;

    Ok(average_duration)
}

pub fn generate_test_cases(
    graph: &dyn Graph,
    number_of_testcases: u32,
) -> Vec<ShortestPathTestCase> {
    (0..)
        .par_bridge()
        .progress_with(get_progressbar_long_jobs(
            "Generation test cases",
            number_of_testcases as u64,
        ))
        .map_init(
            || (thread_rng()),
            |rng, _| {
                let source = rng.gen_range(0..graph.number_of_vertices());
                let target = rng.gen_range(0..graph.number_of_vertices());

                let distance =
                    dijkstra_one_to_one_wrapped(graph, source, target).map(|path| path.distance);

                ShortestPathTestCase {
                    source,
                    target,
                    distance,
                }
            },
        )
        .take_any(number_of_testcases as usize)
        .collect()
}
