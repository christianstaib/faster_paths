use std::collections::HashMap;

use super::contracted_graph::ContractedGraph;
use crate::{
    graphs::{Distance, Graph, Vertex, WeightedEdge},
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataHashMap, Path},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataHashSet},
        },
        shortcuts::replace_shortcuts_slowly,
        PathFinding,
    },
};

impl PathFinding for ContractedGraph {
    fn shortest_path(&self, source: Vertex, target: Vertex) -> Option<Path> {
        one_to_one_wrapped_path(
            self.upward_graph(),
            self.downward_graph(),
            self.shortcuts(),
            source,
            target,
        )
    }

    fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        one_to_one_wrapped_distance(self.upward_graph(), self.downward_graph(), source, target)
    }
}

pub fn get_slow_shortcuts(
    edges_and_predecessors: &Vec<(WeightedEdge, Option<Vertex>)>,
) -> HashMap<(Vertex, Vertex), Vertex> {
    let mut shortcuts: HashMap<(Vertex, Vertex), Vertex> = HashMap::new();

    for (edge, predecessor) in edges_and_predecessors.iter() {
        if let Some(predecessor) = predecessor {
            shortcuts.insert((edge.tail, edge.head), *predecessor);
        }
    }

    shortcuts
}

/// Wrapper that returns the shortest path distance.
pub fn one_to_one_wrapped_distance(
    upward_graph: &dyn Graph,
    downward_graph: &dyn Graph,
    source: Vertex,
    target: Vertex,
) -> Option<Distance> {
    let (_vertex, distance, _forward_data, _backward_data) =
        one_to_one_wrapped(upward_graph, downward_graph, source, target)?;

    Some(distance)
}

/// Wrapper that returns the shortest path.
pub fn one_to_one_wrapped_path(
    upward_graph: &dyn Graph,
    downward_graph: &dyn Graph,
    shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
    source: Vertex,
    target: Vertex,
) -> Option<Path> {
    let (vertex, distance, forward_data, backward_data) =
        one_to_one_wrapped(upward_graph, downward_graph, source, target)?;

    let mut vertices = forward_data.get_path(vertex).unwrap().vertices; // (source -> vertex)
    let mut backward_vertices = backward_data.get_path(vertex).unwrap().vertices; // (target -> vertex)

    backward_vertices.reverse(); // (vertex -> target)
    vertices.pop(); // remove double vertex ((source -> vertex) -> (vertex -> target))
    vertices.extend(backward_vertices); // get (source -> target)

    replace_shortcuts_slowly(&mut vertices, shortcuts); // replace the shortcuts

    Some(Path { vertices, distance })
}

/// Wrapper that returns everythin needed for CH queries.
pub fn one_to_one_wrapped(
    upward_graph: &dyn Graph,
    downward_graph: &dyn Graph,
    source: Vertex,
    target: Vertex,
) -> Option<(Vertex, Distance, DijkstraDataHashMap, DijkstraDataHashMap)> {
    let mut forward_data = DijkstraDataHashMap::new();
    let mut forward_expanded = VertexExpandedDataHashSet::new();
    let mut forward_queue = VertexDistanceQueueBinaryHeap::new();

    let mut backward_data = DijkstraDataHashMap::new();
    let mut backward_expanded = VertexExpandedDataHashSet::new();
    let mut backward_queue = VertexDistanceQueueBinaryHeap::new();

    let (vertex, distance) = one_to_one(
        upward_graph,
        downward_graph,
        &mut forward_data,
        &mut forward_expanded,
        &mut forward_queue,
        &mut backward_data,
        &mut backward_expanded,
        &mut backward_queue,
        source,
        target,
    )?;

    Some((vertex, distance, forward_data, backward_data))
}

/// CH search logic
pub fn one_to_one(
    upward_graph: &dyn Graph,
    downward_graph: &dyn Graph,
    forward_data: &mut dyn DijkstraData,
    forward_expanded: &mut dyn VertexExpandedData,
    forward_queue: &mut dyn VertexDistanceQueue,
    backward_data: &mut dyn DijkstraData,
    backward_expanded: &mut dyn VertexExpandedData,
    backward_queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
    target: Vertex,
) -> Option<(Vertex, Distance)> {
    forward_data.set_distance(source, 0);
    forward_queue.insert(source, 0);

    backward_data.set_distance(target, 0);
    backward_queue.insert(target, 0);

    let mut meeting_vertex_and_distance = None;

    println!("new");
    while !forward_queue.is_empty() || !backward_queue.is_empty() {
        println!("{:?} {:?}", forward_queue.peek(), backward_queue.peek());
        single_search_step(
            upward_graph,
            downward_graph,
            forward_data,
            forward_expanded,
            forward_queue,
            backward_data,
            &mut meeting_vertex_and_distance,
        );

        single_search_step(
            downward_graph,
            upward_graph,
            backward_data,
            backward_expanded,
            backward_queue,
            forward_data,
            &mut meeting_vertex_and_distance,
        );
    }

    meeting_vertex_and_distance
}

/// Single search step in one direction.
fn single_search_step(
    direction1_graph: &dyn Graph,
    direction2_graph: &dyn Graph,
    direction1_data: &mut dyn DijkstraData,
    direction1_expanded: &mut dyn VertexExpandedData,
    direction1_queue: &mut dyn VertexDistanceQueue,
    direction2_data: &mut dyn DijkstraData,
    meeting_vertex_and_distance: &mut Option<(Vertex, Distance)>,
) {
    if let Some((tail, distance_tail)) = direction1_queue.pop() {
        // It is not guaranteed that the queue does implement a decrease key operation.
        // Therefor, if a vertex has already been expanded, skip it.
        if direction1_expanded.expand(tail) {
            return;
        }

        // Stall on demand logic.
        for direction2_edge in direction2_graph.edges(tail) {
            if let Some(predecessor_weight) = direction1_data.get_distance(direction2_edge.head) {
                if predecessor_weight + direction2_edge.weight < distance_tail {
                    return;
                }
            }
        }

        // Meeting vertex logic
        if let Some(direction2_distance_tail) = direction2_data.get_distance(tail) {
            let current_meeting_distance = meeting_vertex_and_distance
                .map(|(_vertex, distance)| distance)
                .unwrap_or(Distance::MAX);
            let alternative_meeting_distance = distance_tail + direction2_distance_tail;
            if alternative_meeting_distance < current_meeting_distance {
                *meeting_vertex_and_distance = Some((tail, alternative_meeting_distance));
            }
        }

        // Search logic
        for edge in direction1_graph.edges(tail) {
            let current_distance_head = direction1_data
                .get_distance(edge.head)
                .unwrap_or(Distance::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                direction1_data.set_distance(edge.head, alternative_distance_head);
                direction1_data.set_predecessor(edge.head, tail);
                direction1_queue.insert(edge.head, alternative_distance_head);
            }
        }
    }
}
