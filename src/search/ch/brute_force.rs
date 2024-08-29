use std::collections::{HashMap, HashSet};

use indicatif::{ParallelProgressIterator, ProgressBar};
use itertools::Itertools;
use rand::prelude::*;
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
    progress_bar: ProgressBar,
) -> (Vec<WeightedEdge>, HashMap<(Vertex, Vertex), Vertex>) {
    // Shuffle vertices for better prediction of remaining time
    let mut vertices = graph.vertices().collect_vec();
    vertices.shuffle(&mut thread_rng());

    let mut all_edges = Vec::new();
    let mut all_shortcuts = HashMap::new();
    let edges_and_shortcuts = vertices
        .into_par_iter()
        .progress_with(progress_bar)
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

pub fn create_shortcuts(
    path: &[Vertex],
    vertex_to_level: &[Level],
) -> Vec<((Vertex, Vertex), Vertex)> {
    assert!(path.len() >= 2);

    if path.len() == 2 {
        return Vec::new();
    }

    let max_level_vertex_index = path
        .iter()
        .enumerate()
        .skip(1) // Skip the first element.
        .take(path.len() - 2) // Take all but the last element.
        .max_by_key(|&(_index, &vertex)| vertex_to_level[vertex as usize])
        .unwrap()
        .0;

    let mut shortcuts = vec![(
        (*path.first().unwrap(), *path.last().unwrap()),
        path[max_level_vertex_index],
    )];

    shortcuts.extend(create_shortcuts(
        &path[..=max_level_vertex_index],
        vertex_to_level,
    ));
    shortcuts.extend(create_shortcuts(
        &path[max_level_vertex_index..],
        vertex_to_level,
    ));

    shortcuts
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

                    let path = data.get_path(shortcut_head).unwrap().vertices;
                    shortcuts.extend(create_shortcuts(&path, vertex_to_level));
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
