use std::{
    cmp::Reverse,
    collections::HashSet,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

use indicatif::*;
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    graphs::{Graph, Level, Vertex},
    search::{
        ch::brute_force::get_ch_edges_wrapped, dijkstra::dijkstra_one_to_one_path_wrapped,
        hl::half_hub_graph::get_hub_label_with_brute_force_wrapped, path::ShortestPathTestCase,
        PathFinding,
    },
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

/// Computes paths of len > 2 in a graph.
pub fn get_paths(
    pathfinder: &dyn PathFinding,
    vertices: &Vec<Vertex>,
    number_of_searches: u32,
    min_len: usize,
    max_len: usize,
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
        .filter(|path| path.vertices.len() >= min_len && path.vertices.len() <= max_len)
        .take_any(number_of_searches as usize)
        .progress_with(pb)
        .map(|mut path| {
            path.vertices.shrink_to_fit();
            path.vertices
        })
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

/// Calculates how many paths a given a level (and therfore a vertex) hits in %.
pub fn hit_percentage(paths: &Vec<Vec<Vertex>>, level_to_vertex: &Vec<Vertex>) -> Vec<f32> {
    let mut hit_percentage = Vec::new();
    let mut active_paths = paths.iter().collect_vec();

    let pb = get_progressbar("Getting hit percentages", level_to_vertex.len() as u64);

    // highest level first
    for &hitting_vertex in level_to_vertex.iter().rev().progress_with(pb) {
        active_paths = active_paths
            .into_par_iter()
            .filter(|path| !path.contains(&hitting_vertex))
            .collect();

        hit_percentage.push((paths.len() - active_paths.len()) as f32 / paths.len() as f32)
    }

    hit_percentage
}

pub fn average_ch_vertex_degree(
    graph: &dyn Graph,
    vertex_to_level: &Vec<Level>,
    num_vertices: u32,
) -> f32 {
    let vertices = graph.non_trivial_vertices();

    let vertices = vertices
        .choose_multiple(&mut thread_rng(), num_vertices as usize)
        .cloned()
        .collect_vec();

    let edges = vertices
        .par_iter()
        .progress_with(get_progressbar("Getting labels", vertices.len() as u64))
        .map(|&vertex| get_ch_edges_wrapped(graph, &vertex_to_level, vertex).0)
        .collect::<Vec<_>>();

    edges.iter().flatten().count() as f32 / edges.len() as f32
}

pub fn average_hl_label_size(
    graph: &dyn Graph,
    vertex_to_level: &Vec<Level>,
    num_labels: u32,
) -> f32 {
    let vertices = graph.non_trivial_vertices();

    let vertices = vertices
        .choose_multiple(&mut thread_rng(), num_labels as usize)
        .cloned()
        .collect_vec();

    let labels = vertices
        .par_iter()
        .progress_with(get_progressbar("Getting labels", vertices.len() as u64))
        .map(|&vertex| get_hub_label_with_brute_force_wrapped(graph, &vertex_to_level, vertex).0)
        .collect::<Vec<_>>();

    labels.iter().flatten().count() as f32 / labels.len() as f32
}

/// Constructs a vector of vertices ordered by their level, where each level is
/// determined by the vertex that hits the most paths. The function processes
/// paths in parallel, iteratively selecting the most frequent vertex and
/// updating active paths until all paths are exhausted.
pub fn hitting_set(paths: &[Vec<Vertex>], number_of_vertices: u32) -> Vec<Vertex> {
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
            .par_iter()
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

    level_to_vertex
}

/// Constructs a vector of vertices ordered by their level, where each level is
/// determined by the vertex that hits the most paths. The function processes
/// paths in parallel, iteratively selecting the most frequent vertex and
/// updating active paths until all paths are exhausted.
pub fn level_to_vertex_with_ord<F, K>(
    paths: &[Vec<Vertex>],
    number_of_vertices: u32,
    sort_hit_first: bool,
    order: F,
) -> Vec<Vertex>
where
    F: Fn(&Vertex) -> K,
    K: Ord,
{
    let mut level_to_vertex = Vec::new();
    let mut active_paths: Vec<usize> = (0..paths.len()).collect();
    let mut active_vertices: HashSet<Vertex> = HashSet::from_iter(0..number_of_vertices);

    let mut all_hits = vec![0; number_of_vertices as usize];

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

        all_hits
            .par_iter_mut()
            .zip(hits.par_iter())
            .for_each(|(all, this)| *all += this);

        // Get the vertex that hits the most paths.
        let vertex = hits
            .par_iter()
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

    let mut active_vertices = active_vertices.into_iter().collect_vec();
    active_vertices.shuffle(&mut thread_rng());

    if sort_hit_first {
        active_vertices.sort_by_cached_key(|vertex| all_hits[*vertex as usize]);
    }

    active_vertices.sort_by_cached_key(|vertex| order(vertex));

    // Insert the remaining vertices at the front, e.g. assign them the lowest
    // levels.
    level_to_vertex.splice(0..0, active_vertices);

    level_to_vertex
}

/// Constructs a vector of vertices ordered by their level, where each level is
/// determined by the vertex that hits the most paths. The function processes
/// paths in parallel, iteratively selecting the most frequent vertex and
/// updating active paths until all paths are exhausted.
pub fn level_to_vertex(paths: &[Vec<Vertex>], number_of_vertices: u32) -> Vec<Vertex> {
    let mut level_to_vertex = Vec::new();
    let mut active_paths: Vec<usize> = (0..paths.len()).collect();
    let mut active_vertices: HashSet<Vertex> = HashSet::from_iter(0..number_of_vertices);

    let mut all_hits = vec![0; number_of_vertices as usize];

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

        all_hits
            .par_iter_mut()
            .zip(hits.par_iter())
            .for_each(|(all, this)| *all += this);

        // Get the vertex that hits the most paths.
        let vertex = hits
            .par_iter()
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

    let mut active_vertices = active_vertices.into_iter().collect_vec();

    active_vertices.sort_by_cached_key(|vertex| all_hits[*vertex as usize]);

    // Insert the remaining vertices at the front, e.g. assign them the lowest
    // levels.
    level_to_vertex.splice(0..0, active_vertices);

    level_to_vertex
}

pub fn benchmark_path(
    pathfinder: &dyn PathFinding,
    sources_and_targets: &[(Vertex, Vertex)],
) -> Duration {
    let pb = get_progressbar("Benchmarking", sources_and_targets.len() as u64);

    let path_and_duration = sources_and_targets
        .into_iter()
        .progress_with(pb)
        .map(|&(source, target)| {
            let start = Instant::now();
            let _path = pathfinder.shortest_path(source, target);
            start.elapsed()
        })
        .collect_vec();

    let average_duration = path_and_duration
        .iter()
        .map(|duration| duration.as_secs_f64())
        .sum::<f64>()
        / path_and_duration.len() as f64;

    Duration::from_secs_f64(average_duration)
}

pub fn benchmark_distances(
    pathfinder: &dyn PathFinding,
    sources_and_targets: &[(Vertex, Vertex)],
) -> Duration {
    let pb = get_progressbar("Benchmarking", sources_and_targets.len() as u64);

    let path_and_duration = sources_and_targets
        .into_iter()
        .progress_with(pb)
        .map(|&(source, target)| {
            let start = Instant::now();
            let _path = pathfinder.shortest_path_distance(source, target);
            start.elapsed()
        })
        .collect_vec();

    let average_duration = path_and_duration
        .iter()
        .map(|duration| duration.as_secs_f64())
        .sum::<f64>()
        / path_and_duration.len() as f64;

    Duration::from_secs_f64(average_duration)
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

pub fn write_json_with_spinnner<T>(name: &str, path: &Path, value: &T)
where
    T: Serialize,
{
    let pb = get_progressspinner(format!("Writing {} as json", name).as_str());
    let writer = BufWriter::new(File::create(path).unwrap());
    serde_json::to_writer(writer, value).unwrap();
    pb.finish_and_clear();
}

pub fn read_json_with_spinnner<T>(name: &str, path: &Path) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let pb = get_progressspinner(format!("Reading {} from json", name).as_str());
    let reader = BufReader::new(File::open(path).unwrap());
    let t = serde_json::from_reader(reader).unwrap();
    pb.finish_and_clear();
    t
}

pub fn write_bincode_with_spinnner<T>(name: &str, path: &Path, value: &T)
where
    T: Serialize,
{
    let pb = get_progressspinner(format!("Writing {} as bincode", name).as_str());
    let writer = BufWriter::new(File::create(path).unwrap());
    bincode::serialize_into(writer, value).unwrap();
    pb.finish_and_clear();
}

pub fn read_bincode_with_spinnner<T>(name: &str, path: &Path) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let pb = get_progressspinner(format!("Reading {} as bincode", name).as_str());
    let reader = BufReader::new(File::open(path).unwrap());
    let t = bincode::deserialize_from(reader).unwrap();
    pb.finish_and_clear();
    t
}
