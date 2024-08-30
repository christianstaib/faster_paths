use std::collections::{HashMap, HashSet};

use indicatif::{ParallelProgressIterator, ProgressBar};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::contracted_graph::{vertex_to_level, ContractedGraph};
use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Level, Vertex, WeightedEdge},
    search::collections::{
        dijkstra_data::{DijkstraData, DijkstraDataVec},
        vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
        vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
    },
    utility::get_progressbar_long_jobs,
};

impl ContractedGraph {
    pub fn by_brute_force<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        level_to_vertex: &Vec<Vertex>,
    ) -> ContractedGraph {
        let vertex_to_level = vertex_to_level(level_to_vertex);

        let (mut edges, mut shortcuts) = brute_force_contracted_graph_edges(
            graph.out_graph(),
            &vertex_to_level,
            get_progressbar_long_jobs(
                "Brute forcing upward edges",
                graph.out_graph().number_of_vertices() as u64,
            ),
        );

        let (downward_edges, downward_shortcuts) = brute_force_contracted_graph_edges(
            graph.in_graph(),
            &vertex_to_level,
            get_progressbar_long_jobs(
                "Brute forcing downward edges",
                graph.in_graph().number_of_vertices() as u64,
            ),
        );

        edges.extend(downward_edges);

        shortcuts.extend(
            downward_shortcuts
                .into_iter()
                .map(|((tail, head), skiped_vertex)| ((head, tail), skiped_vertex)),
        );

        ContractedGraph::new(level_to_vertex.clone(), edges, shortcuts)
    }
}

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

    let mut shortcuts = Vec::new();
    let mut stack = vec![(0, path.len() - 1)];

    while let Some((tail_index, head_index)) = stack.pop() {
        if head_index - tail_index >= 2 {
            // Find the index of the vertex with the maximum level between start_index and
            // end_index
            let skiped_index = path[tail_index + 1..head_index]
                .iter()
                .enumerate()
                .max_by_key(|&(_index, &vertex)| vertex_to_level[vertex as usize])
                .unwrap()
                .0
                + tail_index
                + 1;

            shortcuts.push(((path[tail_index], path[head_index]), path[skiped_index]));

            stack.push((tail_index, skiped_index));
            stack.push((skiped_index, head_index));
        }
    }

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
