use std::collections::{HashMap, HashSet};

use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    graphs::{Distance, Graph, Level, Vertex, WeightedEdge},
    search::collections::{
        dijkstra_data::{DijkstraData, DijkstraDataVec},
        vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
        vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
    },
};

pub fn brute_force_contracted_graph_edges(
    graph: &dyn Graph,
    vertex_to_level: &Vec<u32>,
) -> (Vec<WeightedEdge>, HashMap<(Vertex, Vertex), Vertex>) {
    let mut all_edges = Vec::new();
    let mut all_shortcuts = HashMap::new();
    let edges_and_shortcuts = graph
        .vertices()
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
                let vertex_edges_and_shortcuts =
                    get_ch_edges(graph, data, expanded, queue, vertex_to_level, vertex);

                data.clear();
                expanded.clear();
                queue.clear();

                vertex_edges_and_shortcuts
            },
        )
        .collect::<Vec<_>>();

    for (edges, shortcuts) in edges_and_shortcuts.into_iter() {
        all_edges.extend(edges);
        all_shortcuts.extend(shortcuts);
    }

    (all_edges, all_shortcuts)
}

pub fn get_ch_edges(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    vertex_to_level: &Vec<u32>,
    source: Vertex,
) -> (Vec<WeightedEdge>, Vec<((Vertex, Vertex), Vertex)>) {
    // Maps (vertex -> (max level on path from source to vertex, associated vertex))
    //
    // A vertex is a head of a ch edge if its levels equals the max level on its
    // path from the source. The tail of this ch edge is is the vertex with the
    // max level on the path to the head's predecessor
    let mut max_level: HashMap<Vertex, (Level, Vertex)> = HashMap::new();
    max_level.insert(source, (vertex_to_level[source as usize], source));

    // Keeps track of vertices that potentially could be the head of a ch edge with
    // a tail in source. If there are no more alive vertices, the search can be
    // stopped early.
    let mut alive = HashSet::from([source]);
    alive.insert(source);

    data.set_distance(source, 0);
    queue.insert(source, 0);

    let mut edges = Vec::new();
    let mut shortcuts = Vec::new();

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }
        if alive.is_empty() {
            break;
        }

        let (max_level_tail, max_level_tail_vertex) = max_level[&tail];
        let level_tail = vertex_to_level[tail as usize];

        // Check if tail is a head of a ch edge
        if max_level_tail == level_tail {
            // for less confusion, rename variables

            // Dont create a edge from source to source. source has no predecessor
            if let Some(predecessor) = data.get_predecessor(tail) {
                let shortcut_tail = max_level.get(&predecessor).unwrap().1;
                let shortcut_head = tail;

                // Only add edge if its tail is source. This function only returns edges with a
                // tail in source.
                if shortcut_tail == source {
                    edges.push(WeightedEdge::new(
                        shortcut_tail,
                        shortcut_head,
                        data.get_distance(tail).unwrap(),
                    ));

                    // if let Some(head_predecessor) = data.get_predecessor(shortcut_head) {
                    if let Some(mut path) = data.get_path(shortcut_head) {
                        path.vertices.remove(0);
                        path.vertices.pop();

                        path.vertices
                            .iter()
                            .max_by_key(|&&vertex| vertex_to_level[vertex as usize])
                            .map(|&skiped_vertex| {
                                shortcuts.push(((shortcut_tail, shortcut_head), skiped_vertex))
                            });
                    }
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
                    max_level.insert(edge.head, (level_head, edge.head));
                } else {
                    max_level.insert(edge.head, (max_level_tail, max_level_tail_vertex));
                }

                if tail_is_alive {
                    alive.insert(edge.head);
                }
            }
        }

        alive.remove(&tail);
    }

    (edges, shortcuts)
}
