use std::usize;

use ahash::{HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::Rng;
use rayon::prelude::*;

use super::{
    edge::DirectedWeightedEdge,
    path::{Path, ShortestPathRequest, ShortestPathTestCase},
    vec_graph::VecGraph,
    Graph, VertexId,
};
use crate::{classical_search::dijkstra::Dijkstra, dijkstra_data::DijkstraData};

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

pub fn number_of_edges(graph: &dyn Graph) -> u32 {
    (0..graph.number_of_vertices())
        .map(|vertex| graph.out_edges(vertex).len() as u32)
        .sum::<u32>()
}

pub fn to_vec_graph(graph: &dyn Graph) -> VecGraph {
    VecGraph::from_edges(&all_edges(graph))
}

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

    return true;
}

pub fn hitting_set(paths: &[Path], number_of_vertices: u32) -> Vec<VertexId> {
    let mut hitting_set = Vec::new();
    let mut active_paths: HashSet<usize> = (0..paths.len()).collect();

    let pb = ProgressBar::new(active_paths.len() as u64);
    while !active_paths.is_empty() {
        let mut number_of_hits = vec![0; number_of_vertices as usize];

        for &path_idx in active_paths.iter() {
            let path = &paths[path_idx];
            for &vertex in path.vertices.iter() {
                number_of_hits[vertex as usize] += 1;
            }
        }

        let max_hitting_vertex = number_of_hits
            .iter()
            .enumerate()
            .max_by_key(|(_, &hits)| hits)
            .unwrap()
            .0;
        hitting_set.push(max_hitting_vertex as VertexId);

        active_paths.retain(|&paths_idx| {
            !paths[paths_idx]
                .vertices
                .contains(&(max_hitting_vertex as VertexId))
        });

        pb.set_position((paths.len() - active_paths.len()) as u64);
    }
    pb.finish();

    hitting_set
}

pub fn test_cases(number_of_paths: u32, graph: &dyn Graph) -> Vec<ShortestPathTestCase> {
    let dijkstra = Dijkstra::new(graph);

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

                let data = dijkstra.get_data(request.source(), request.target());
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

pub fn random_paths(number_of_paths: u32, graph: &dyn Graph) -> Vec<Path> {
    let dijkstra = Dijkstra::new(graph);

    (0..u32::MAX)
        .into_par_iter()
        .take(number_of_paths as usize)
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

                let data = dijkstra.get_data(request.source(), request.target());
                data.get_path(target)
            },
        )
        .flatten()
        .collect()
}
