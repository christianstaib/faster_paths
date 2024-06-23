use std::{
    cmp::Reverse,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration, Instant},
    usize,
};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use super::{
    edge::DirectedWeightedEdge,
    path::{
        Path, PathFinding, ShortestPathRequest, ShortestPathTestCase, ShortestPathTestTimingResult,
    },
    Graph, VertexId,
};
use crate::{
    classical_search::dijkstra::{get_data, Dijkstra},
    dijkstra_data::{dijkstra_data_vec::DijkstraDataVec, DijkstraData},
    graphs::{edge::DirectedEdge, reversible_vec_graph::ReversibleVecGraph},
    hl::{
        hl_from_top_down::{generate_forward_label, generate_reverse_label},
        pathfinding::shortest_path,
    },
    shortcut_replacer::slow_shortcut_replacer::replace_shortcuts_slow,
};

/// Check if a route is correct for a given request. Panics if not.
pub fn validate_path(
    graph: &dyn Graph,
    validation: &ShortestPathTestCase,
    path: &Option<Path>,
) -> Result<(), String> {
    if let Some(path) = path {
        if let Some(weight) = validation.weight {
            if path.weight != weight {
                return Err("wrong path weight".to_string());
            }

            // Ensure that path is not empty when it should not be.
            if path.vertices.is_empty()
                && validation.request.source() != validation.request.target()
            {
                return Err("path is empty".to_string());
            }

            // Ensure fist and last vertex of path are source and target of request.
            if let Some(first_vertex) = path.vertices.first() {
                if first_vertex != &validation.request.source() {
                    return Err("first vertex of path is not source of request".to_string());
                }
            }
            if let Some(last_vertex) = path.vertices.last() {
                if last_vertex != &validation.request.target() {
                    return Err("last vertex of path is not target of request".to_string());
                }
            }

            // check if there is an edge between consecutive path vertices.
            let mut edges = Vec::new();
            for index in 0..(path.vertices.len() - 1) {
                let tail = path.vertices[index];
                let head = path.vertices[index + 1];
                if let Some(min_edge) = graph.out_edges(tail).find(|edge| edge.head() == head) {
                    edges.push(min_edge);
                } else {
                    return Err(format!("no edge between {} and {} found", tail, head));
                }
            }

            // check if total weight of path is correct.
            let true_cost = edges.iter().map(|edge| edge.weight()).sum::<u32>();
            if path.weight != true_cost || path.weight != weight {
                return Err("wrong path weight".to_string());
            }
        } else {
            return Err("a path was found where there should be none".to_string());
        }
    } else if validation.weight.is_some() {
        return Err("no path is found but there should be one".to_string());
    }

    Ok(())
}

pub fn all_edges(graph: &dyn Graph) -> Vec<DirectedWeightedEdge> {
    (0..graph.number_of_vertices())
        .flat_map(|vertex| graph.out_edges(vertex))
        .collect()
}

pub fn change_representation<T>(graph: &dyn Graph) -> T
where
    T: Graph + Default,
{
    let mut new_graph = T::default();

    for vertex in 0..graph.number_of_vertices() {
        for edge in graph.out_edges(vertex) {
            new_graph.set_edge(&edge);
        }
    }

    new_graph
}

pub fn number_of_edges(graph: &dyn Graph) -> u32 {
    (0..graph.number_of_vertices())
        .map(|vertex| graph.out_edges(vertex).len() as u32)
        .sum::<u32>()
}

// pub fn to_vec_graph(graph: &dyn Graph) -> VecGraph {
// }

pub fn neighbors(vertex: VertexId, graph: &dyn Graph) -> HashSet<VertexId> {
    let mut neighbors = HashSet::new();

    for out_edge in graph.out_edges(vertex) {
        neighbors.insert(out_edge.head());
    }

    for in_edge in graph.in_edges(vertex) {
        neighbors.insert(in_edge.tail());
    }

    neighbors
}

pub fn add_edge_bidrectional(graph: &mut dyn Graph, edge: &DirectedWeightedEdge) {
    graph.set_edge(edge);
    graph.set_edge(&edge.reversed());
}

pub fn is_bidirectional(graph: &dyn Graph) -> bool {
    for vertex in 0..graph.number_of_vertices() {
        for out_edge in graph.out_edges(vertex) {
            if graph.get_edge_weight(&out_edge.reversed().unweighted()) != Some(out_edge.weight()) {
                return false;
            }
        }
    }

    true
}

pub fn hitting_set(paths: &[Path], number_of_vertices: u32) -> (Vec<VertexId>, Vec<u32>) {
    let mut hitting_set = Vec::new();
    let mut active_paths: Vec<usize> = (0..paths.len()).collect();

    let all_hits = (0..number_of_vertices)
        .map(|_| AtomicU32::new(0))
        .collect_vec();

    let pb = ProgressBar::new(active_paths.len() as u64);
    while !active_paths.is_empty() {
        let number_of_hits = (0..number_of_vertices)
            .map(|_| AtomicU32::new(0))
            .collect_vec();

        active_paths.par_iter().for_each(|&path_idx| {
            let path = &paths[path_idx];
            for &vertex in path.vertices.iter() {
                number_of_hits[vertex as usize].fetch_add(1, Ordering::Relaxed);
                all_hits[vertex as usize].fetch_add(1, Ordering::Relaxed);
            }
        });

        let max_hitting_vertex = number_of_hits
            .iter()
            .enumerate()
            .max_by_key(|(_, hits)| hits.load(Ordering::Acquire))
            .unwrap()
            .0;
        hitting_set.push(max_hitting_vertex as VertexId);

        active_paths = active_paths
            .into_par_iter()
            .filter(|&paths_idx| {
                !paths[paths_idx]
                    .vertices
                    .contains(&(max_hitting_vertex as VertexId))
            })
            .collect();

        pb.set_position((paths.len() - active_paths.len()) as u64);
    }
    pb.finish();

    (
        hitting_set,
        all_hits
            .iter()
            .map(|hits| hits.load(Ordering::Acquire))
            .collect(),
    )
}

pub fn generate_random_pair_testcases(
    number_of_paths: u32,
    graph: &dyn Graph,
) -> Vec<ShortestPathTestCase> {
    (0..number_of_paths)
        .into_par_iter()
        .progress()
        .map_init(
            rand::thread_rng, // get the thread-local RNG
            |rng, _| {
                // guarantee that source != tatget.
                let source = rng.gen_range(0..graph.number_of_vertices());
                let mut target = rng.gen_range(0..graph.number_of_vertices() - 1);
                if target >= source {
                    target += 1;
                }

                let request = ShortestPathRequest::new(source, target).unwrap();

                let data = get_data(graph, request.source(), request.target());
                let path = data.get_path(target);

                let mut weight = None;
                if let Some(path) = path {
                    weight = Some(path.weight);
                }

                ShortestPathTestCase {
                    request,
                    weight,
                    dijkstra_rank: data.dijkstra_rank(),
                }
            },
        )
        .collect()
}

pub fn random_paths(
    path_finder: &dyn PathFinding,
    number_of_paths: u32,
    number_of_vertices: u32,
    max_seconds: u64,
) -> Vec<Path> {
    let start = Instant::now();
    (0..u32::MAX)
        .take_while(|_| start.elapsed().as_secs() < max_seconds)
        .par_bridge()
        .map_init(
            rand::thread_rng, // get the thread-local RNG
            |rng, _| {
                // return None if no valid request can be build
                let request = {
                    // guarantee that source != target
                    let source = rng.gen_range(0..number_of_vertices);
                    let mut target = rng.gen_range(0..number_of_vertices - 1);
                    if target >= source {
                        target += 1;
                    }

                    ShortestPathRequest::new(source, target)
                }?;

                path_finder.shortest_path(&request)
            },
        )
        .flatten() // flatten Option<Path> to Path
        .take_any(number_of_paths as usize)
        .progress_count(number_of_paths as u64)
        .collect()
}

pub fn degree_vec(graph: &dyn Graph) -> Vec<u32> {
    (0..graph.number_of_vertices())
        .map(|vertex| graph.out_edges(vertex).len() as u32)
        .collect()
}

pub fn random_request(graph: &dyn Graph, rng: &mut ThreadRng) -> Option<ShortestPathRequest> {
    if graph.number_of_vertices() <= 1 {
        // not enough vertices to get a request with source != target
        return None;
    }

    // guarantee that source != target
    let source = rng.gen_range(0..graph.number_of_vertices());
    let mut target = rng.gen_range(0..graph.number_of_vertices() - 1);
    if target >= source {
        target += 1;
    }

    ShortestPathRequest::new(source, target)
}

pub fn shortests_path_tree(data: &DijkstraDataVec) -> Vec<Vec<VertexId>> {
    let mut search_tree = vec![Vec::new(); data.vertices.len()];

    for (child, entry) in data.vertices.iter().enumerate() {
        if let Some(parent) = entry.predecessor {
            search_tree[parent as usize].push(child as VertexId);
        }
    }

    search_tree
}

pub fn validate_and_time(
    test_cases: &[ShortestPathTestCase],
    path_finder: &dyn PathFinding,
    graph: &dyn Graph,
) -> (Duration, Vec<ShortestPathTestTimingResult>) {
    let mut times = Vec::new();

    let mut paths = Vec::new();
    println!("Timing");
    test_cases.iter().progress().for_each(|test_case| {
        let start = Instant::now();
        let path = path_finder.shortest_path(&test_case.request);
        let duration = start.elapsed();
        let timing_result = ShortestPathTestTimingResult {
            test_case: test_case.clone(),
            timing_in_seconds: duration.as_secs_f64(),
        };
        times.push(timing_result);

        paths.push(path);
    });

    println!("Validating");
    for i in (0..test_cases.len()).progress() {
        if let Err(err) = validate_path(graph, &test_cases[i], &paths[i]) {
            panic!("top down hl wrong: {}", err);
        }
    }

    let average: f64 = times
        .iter()
        .map(|result| result.timing_in_seconds)
        .sum::<f64>()
        / times.len() as f64;

    (Duration::from_secs_f64(average), times)
}

pub fn generate_random_pair_test_cases(
    graph: &dyn Graph,
    number_of_testcases: u32,
) -> Vec<ShortestPathTestCase> {
    (0..number_of_testcases)
        .progress()
        .par_bridge()
        .map_init(
            rand::thread_rng, // get the thread-local RNG
            |rng, _| {
                // guarantee that source != tatget.
                let source = rng.gen_range(0..graph.number_of_vertices());
                let mut target = rng.gen_range(0..graph.number_of_vertices() - 1);
                if target >= source {
                    target += 1;
                }

                let request = ShortestPathRequest::new(source, target).unwrap();

                let data = get_data(graph, request.source(), request.target());
                let path = data.get_path(target);

                let mut weight = None;
                if let Some(path) = path {
                    weight = Some(path.weight);
                }

                ShortestPathTestCase {
                    request,
                    weight,
                    dijkstra_rank: data.dijkstra_rank(),
                }
            },
        )
        .collect()
}

/// Retruns a vec where \[v\] is the level of a vertex v
pub fn generate_hiting_set_order_with_hub_labels(
    number_of_hubs: u32,
    graph: &dyn Graph,
) -> Vec<u32> {
    let mut order = (0..graph.number_of_vertices()).collect_vec();
    order.shuffle(&mut thread_rng());

    let shortcuts: Arc<DashMap<DirectedEdge, VertexId>> = Arc::new(DashMap::new());

    let vertices =
        (0..graph.number_of_vertices()).choose_multiple(&mut thread_rng(), number_of_hubs as usize);
    let forward: Vec<_> = vertices
        .par_iter()
        .progress()
        .map(|&vertex| {
            let (label, label_shortcuts) = generate_forward_label(vertex, graph, &order);

            for (edge, vertex_id) in label_shortcuts {
                // DashMap's entry API can be used to efficiently check and update the map
                shortcuts.entry(edge).or_insert(vertex_id);
            }

            (vertex, label)
        })
        .collect();

    let reverse: Vec<_> = vertices
        .par_iter()
        .progress()
        .map(|&vertex| {
            let (label, label_shortcuts) = generate_reverse_label(vertex, graph, &order);

            for (edge, vertex_id) in label_shortcuts {
                // DashMap's entry API can be used to efficiently check and update the map
                shortcuts.entry(edge).or_insert(vertex_id);
            }

            (vertex, label)
        })
        .collect();

    let shortcuts = Arc::into_inner(shortcuts).unwrap().into_iter().collect();

    let pb = ProgressBar::new((vertices.len().pow(2) - vertices.len()) as u64);
    let paths: Vec<_> = forward
        .par_iter()
        .flat_map(|(vertex, forward_label)| {
            reverse
                .iter()
                .filter(|(reverse_vertex, _)| reverse_vertex != vertex)
                .filter_map(|(_, reverse_label)| {
                    pb.inc(1);
                    if let Some(mut path) = shortest_path(forward_label, reverse_label) {
                        replace_shortcuts_slow(&mut path.vertices, &shortcuts);
                        return Some(path);
                    }
                    None
                })
                .collect_vec()
        })
        .collect();
    pb.finish();

    println!("generating hitting set");
    let (mut hitting_setx, num_hits) = hitting_set(&paths, graph.number_of_vertices());

    println!("prepare vertex order");
    let mut not_hitting_set = (0..graph.number_of_vertices())
        .filter(|vertex| !hitting_setx.contains(vertex))
        .collect_vec();

    // shuffle to break neighboring ties
    not_hitting_set.shuffle(&mut thread_rng());
    not_hitting_set.sort_unstable_by_key(|&vertex| Reverse(num_hits[vertex as usize]));

    hitting_setx.extend(not_hitting_set);
    hitting_setx.reverse();

    // hitting_setx maps (order -> vertex)
    // order maps (vertex -> order)

    println!("generate position map");
    // Create a HashMap for quick lookup of positions in hitting_setx
    let position_map: HashMap<u32, usize> = hitting_setx
        .iter()
        .progress()
        .enumerate()
        .map(|(idx, &vertex)| (vertex, idx))
        .collect();

    println!("generate order");
    let order: Vec<u32> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            // Use expect to handle the case where vertex is not found
            *position_map
                .get(&vertex)
                .expect("Vertex not found in hitting_setx") as u32
        })
        .collect();
    order
}

/// Retruns a vec where \[v\] is the level of a vertex v
pub fn generate_hiting_set_order(number_of_random_pairs: u32, graph: &dyn Graph) -> Vec<u32> {
    println!("Generating {} random paths", number_of_random_pairs);
    let number_of_vertices = graph.number_of_vertices();
    let graph = ReversibleVecGraph::from_edges(&all_edges(graph));
    let dijkstra = Dijkstra {
        graph: Box::new(graph),
    };
    let paths = random_paths(
        &dijkstra,
        number_of_random_pairs,
        number_of_vertices,
        u64::MAX,
    );

    println!("generating hitting set");
    let (mut hitting_setx, num_hits) = hitting_set(&paths, number_of_vertices);

    println!("generating vertex order");
    let mut not_hitting_set = (0..number_of_vertices)
        .filter(|vertex| !hitting_setx.contains(vertex))
        .collect_vec();

    // shuffle to break neighboring ties
    not_hitting_set.shuffle(&mut thread_rng());
    not_hitting_set.sort_unstable_by_key(|&vertex| Reverse(num_hits[vertex as usize]));

    hitting_setx.extend(not_hitting_set);
    hitting_setx.reverse();

    let order: Vec<_> = (0..number_of_vertices)
        .into_par_iter()
        .map(|vertex| hitting_setx.iter().position(|&x| x == vertex).unwrap() as u32)
        .collect();
    order
}
