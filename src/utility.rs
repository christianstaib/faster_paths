use std::{
    collections::HashSet,
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

use indicatif::*;
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    graphs::{Graph, Vertex},
    search::{dijkstra::dijkstra_one_to_one_path_wrapped, path::ShortestPathTestCase, PathFinding},
};

pub fn get_progressbar(job_name: &str, len: u64) -> ProgressBar {
    let bar = ProgressBar::new(len);
    bar.set_message(job_name.to_string());
    bar.set_style(
        ProgressStyle::with_template(
            "{msg} {wide_bar} ({percent_precise}%) estimated remaining: {eta_precise}",
        )
        .unwrap(),
    );
    bar
}

pub fn get_progressspinner(job_name: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message(job_name.to_string());
    bar.set_style(ProgressStyle::with_template("{msg} {spinner}").unwrap());
    bar
}

/// Computes paths in a graph using Dijkstra's algorithm.
pub fn get_paths(
    pathfinder: &dyn PathFinding,
    vertices: &Vec<Vertex>,
    number_of_searches: u32,
) -> Vec<Vec<Vertex>> {
    let pb = get_progressbar("Getting many paths", number_of_searches as u64);

    (0..)
        .par_bridge()
        .map_init(
            || thread_rng(),
            |rng, _| {
                let (source, target) = vertices
                    .choose_multiple(rng, 2)
                    .cloned()
                    .collect_tuple()
                    .unwrap();

                pathfinder.shortest_path(source, target)
            },
        )
        .flatten()
        .take_any(number_of_searches as usize)
        .progress_with(pb)
        .map(|path| path.vertices)
        .collect()
}

pub fn get_paths_large(
    pathfinder: &dyn PathFinding,
    vertices: &Vec<Vertex>,
    number_of_vertices: u64,
) -> Vec<Vec<Vertex>> {
    let pb = get_progressbar("Getting many paths", number_of_vertices);

    let curr_num_vertices = AtomicU64::new(0);

    let paths = (0..)
        .par_bridge()
        .map_init(
            || thread_rng(),
            |rng, _| {
                let (source, target) = vertices
                    .choose_multiple(rng, 2)
                    .cloned()
                    .collect_tuple()
                    .unwrap();

                pathfinder.shortest_path(source, target)
            },
        )
        .flatten()
        .take_any_while(|path| {
            let old = curr_num_vertices.fetch_add(
                path.vertices.len() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
            pb.inc(old);
            old < number_of_vertices
        })
        .map(|path| path.vertices)
        .collect();
    pb.finish_and_clear();

    paths
}

/// Constructs a vector of vertices ordered by their level, where each level is
/// determined by the vertex that hits the most paths. The function processes
/// paths in parallel, iteratively selecting the most frequent vertex and
/// updating active paths until all paths are exhausted.
pub fn level_to_vertex(paths: &[Vec<Vertex>], number_of_vertices: u32) -> Vec<Vertex> {
    let mut level_to_vertex = Vec::new();
    let mut active_paths: Vec<usize> = (0..paths.len()).collect();
    let mut active_vertices: HashSet<Vertex> = HashSet::from_iter(0..number_of_vertices);

    let pb = get_progressbar("Generating level_to_vertex vector", paths.len() as u64);
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

pub fn benchmark(
    pathfinder: &dyn PathFinding,
    sources_and_targets: &[(Vertex, Vertex)],
) -> Duration {
    let pb = get_progressbar("Benchmarking", sources_and_targets.len() as u64);

    let path_and_duration = sources_and_targets
        .into_iter()
        .progress_with(pb)
        .map(|&(source, target)| {
            let start = Instant::now();
            let path = pathfinder.shortest_path(source, target);
            (path, start.elapsed())
        })
        .collect_vec();

    let average_duration = path_and_duration
        .iter()
        .map(|(_path, duration)| duration)
        .sum::<Duration>()
        / path_and_duration.len() as u32;

    average_duration
}

/// Non trivial unequal pairs
pub fn gen_tests_cases(graph: &dyn Graph, num: u32) -> Vec<(u32, u32)> {
    let mut rng = thread_rng();
    let non_trivial_vertices = graph.non_trivial_vertices();

    (0..num)
        .map(|_| {
            let source_and_target = non_trivial_vertices
                .choose_multiple(&mut rng, 2)
                .collect_vec();
            (*source_and_target[0], *source_and_target[1])
        })
        .collect_vec()
}

pub fn benchmark_and_test_distance(
    tests: &[ShortestPathTestCase],
    pathfinder: &dyn PathFinding,
) -> Result<Duration, String> {
    // First only calculate paths. Checking correctness immediately would poison
    // cache.
    let path_and_duration = tests
        .into_iter()
        .progress_with(get_progressbar("Benchmarking", tests.len() as u64))
        .map(|test| {
            let start = Instant::now();
            let path = pathfinder.shortest_path(test.source, test.target);
            (path, start.elapsed())
        })
        .collect_vec();

    for (test, (path, _duration)) in tests
        .iter()
        .zip(path_and_duration.iter())
        .progress_with(get_progressbar("Validating", tests.len() as u64))
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
    }

    let average_duration = path_and_duration
        .iter()
        .map(|(_path, duration)| duration)
        .sum::<Duration>()
        / path_and_duration.len() as u32;

    Ok(average_duration)
}

pub fn benchmark_and_test_path(
    graph: &dyn Graph,
    tests: &[ShortestPathTestCase],
    pathfinder: &dyn PathFinding,
) -> Result<Duration, String> {
    // First only calculate paths. Checking correctness immediately would poison
    // cache.
    let path_and_duration = tests
        .into_iter()
        .progress_with(get_progressbar("Benchmarking", tests.len() as u64))
        .map(|test| {
            let start = Instant::now();
            let path = pathfinder.shortest_path(test.source, test.target);
            (path, start.elapsed())
        })
        .collect_vec();

    for (test, (path, _duration)) in tests
        .iter()
        .zip(path_and_duration.iter())
        .progress_with(get_progressbar("Validating", tests.len() as u64))
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
    let non_trivial_vertices = graph.non_trivial_vertices();

    (0..)
        .par_bridge()
        .progress_with(get_progressbar(
            "Generation test cases",
            number_of_testcases as u64,
        ))
        .map_init(
            || (thread_rng()),
            |rng, _| {
                let source_and_target = non_trivial_vertices.choose_multiple(rng, 2).collect_vec();
                let source = *source_and_target[0];
                let target = *source_and_target[1];

                let distance = dijkstra_one_to_one_path_wrapped(graph, source, target)
                    .map(|path| path.distance);

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
