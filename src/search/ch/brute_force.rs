use std::collections::{HashMap, HashSet};

use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::contracted_graph::{vertex_to_level, ContractedGraph};
use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
        WeightedEdge,
    },
    search::collections::{
        dijkstra_data::{DijkstraData, DijkstraDataVec},
        vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
        vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
    },
};

pub fn get_ch_edges(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    vertex_to_level: &Vec<u32>,
    source: Vertex,
) -> Vec<WeightedEdge> {
    // Maps (vertex -> (max level on path from source to vertex, associated vertex))
    //
    // A vertex is a head of a ch edge if its levels equals the max level on its
    // path from the source. The tail of this ch edge is is the vertex with the
    // max level on the path to the head's predecessor
    let mut max_level_on_path = HashMap::new();
    max_level_on_path.insert(source, (vertex_to_level[source as usize], source));

    // Keeps track of vertices that potentially could be the head of a ch edge with
    // a tail in source. If there are no more alive vertices, the search can be
    // stopped early.
    let mut alive = HashSet::from([source]);
    alive.insert(source);

    data.set_distance(source, 0);
    queue.insert(source, 0);

    let mut edges = Vec::new();

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }
        if alive.is_empty() {
            // break;
        }

        let (max_level_tail, max_level_tail_vertex) = max_level_on_path[&tail];
        let level_tail = vertex_to_level[tail as usize];

        // Check if tail is a head of a ch edge
        if max_level_tail == level_tail {
            // Dont create a edge from source to source
            if tail != source {
                let predecessor = data.get_predecessor(tail).unwrap();
                let edge_tail = max_level_on_path.get(&predecessor).unwrap().1;

                // Only add edge if its tail is source. This function only returns edges with a
                // tail in source.
                if edge_tail == source {
                    edges.push(WeightedEdge::new(
                        edge_tail,
                        tail,
                        data.get_distance(tail).unwrap(),
                    ));
                }
                alive.remove(&tail);
            }
        }

        let tail_is_alive = alive.contains(&tail);

        for edge in graph.edges(tail) {
            let current_distance_head = data.get_distance(edge.head).unwrap_or(Distance::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                data.set_distance(edge.head, alternative_distance_head);
                data.set_predecessor(edge.head, tail);
                queue.insert(edge.head, alternative_distance_head);

                let level_head = vertex_to_level[edge.head as usize];
                if level_head > max_level_tail {
                    max_level_on_path.insert(edge.head, (level_head, edge.head));
                } else {
                    max_level_on_path.insert(edge.head, (max_level_tail, max_level_tail_vertex));
                }

                if tail_is_alive {
                    alive.insert(edge.head);
                }
            }
        }

        alive.remove(&tail);
    }

    edges
}

pub fn brute_force_contracted_graph(
    graph: &ReversibleGraph<VecVecGraph>,
    level_to_vertex: &Vec<u32>,
) -> ContractedGraph {
    let vertex_to_level = vertex_to_level(level_to_vertex);

    let upward_edges = brute_force_contracted_graph_edges(graph.out_graph(), &vertex_to_level);
    let upward_graph = VecVecGraph::from_edges(&upward_edges);

    let downard_edges = brute_force_contracted_graph_edges(graph.in_graph(), &vertex_to_level);
    let downward_graph = VecVecGraph::from_edges(&downard_edges);

    ContractedGraph {
        upward_graph,
        downward_graph,
        level_to_vertex: level_to_vertex.clone(),
        vertex_to_level,
    }
}

fn brute_force_contracted_graph_edges(
    graph: &dyn Graph,
    vertex_to_level: &Vec<u32>,
) -> Vec<WeightedEdge> {
    (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map_init(
            || {
                (
                    DijkstraDataVec::new(graph),
                    VertexExpandedDataBitSet::new(graph),
                    VertexDistanceQueueBinaryHeap::new(),
                )
            },
            |(data, expanded, queue), vertex| {
                let edges = get_ch_edges(graph, data, expanded, queue, vertex_to_level, vertex);

                data.clear();
                expanded.clear();
                queue.clear();

                edges
            },
        )
        .flatten()
        .collect::<Vec<_>>()
}
