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

pub fn get_ch_edges_wrapped(
    graph: &dyn Graph,
    vertex_to_level: &Vec<u32>,
    source: Vertex,
) -> Vec<WeightedEdge> {
    let mut data = DijkstraDataVec::new(graph);
    let mut expanded = VertexExpandedDataBitSet::new(graph);
    let mut queue = VertexDistanceQueueBinaryHeap::new();
    get_ch_edges(
        graph,
        &mut data,
        &mut expanded,
        &mut queue,
        vertex_to_level,
        source,
    )
}

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

// #[cfg(test)]
// mod tests {
//     use std::{collections::HashSet, ops::Sub};
//
//     use crate::{
//         graphs::{large_test_graph, Graph},
//         search::{
//             ch::{brute_force::get_ch_edges_wrapped,
// large_test_contracted_graph},             
// dijkstra::dijkstra_one_to_one_wrapped,         },
//     };
//
//     #[test]
//     fn brute_force_contracted_graph() {
//         let (graph, _test_cases) = large_test_graph();
//         let contracted_graph = large_test_contracted_graph();
//
//         half_brute_force_contracted_graph(
//             graph.out_graph(),
//             &contracted_graph.upward_graph,
//             &contracted_graph.vertex_to_level,
//         );
//
//         half_brute_force_contracted_graph(
//             graph.in_graph(),
//             &contracted_graph.downward_graph,
//             &contracted_graph.vertex_to_level,
//         );
//     }
//
//     fn half_brute_force_contracted_graph(
//         graph: &dyn Graph,
//         contracted_graph: &dyn Graph,
//         vertex_to_level: &Vec<u32>,
//     ) {
//         for vertex in 0..graph.number_of_vertices() {
//             let ch_edges =
// contracted_graph.edges(vertex).collect::<HashSet<_>>();
//
//             let brute_force_ch_edges = get_ch_edges_wrapped(graph,
// vertex_to_level, vertex)                 .into_iter()
//                 .collect::<HashSet<_>>();
//
//             // The brute force edges are the minimal ammount of edges, it is
// no problem if             // there are more ch edges.
//             assert!(brute_force_ch_edges.is_subset(&ch_edges));
//
//             // But for all ch edges that are not brute force edges, we need
// to prove, that             // they are unnecessary.
//             for ch_edge in ch_edges.sub(&brute_force_ch_edges) {
//                 let vertices = dijkstra_one_to_one_wrapped(graph, vertex,
// ch_edge.head)                     .unwrap()
//                     .vertices;
//
//                 assert!(vertices.iter().any(|&vertex| vertex ==
// ch_edge.head));             }
//         }
//     }
// }
