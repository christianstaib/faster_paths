use std::collections::HashMap;

use indicatif::ProgressIterator;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::{
    brute_force::brute_force_contracted_graph_edges, contraction::contraction_with_witness_search,
    contraction_heuristic::contraction_with_heuristic,
};
use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_graph::VecGraph, Distance, Graph, Level, Vertex,
        WeightedEdge,
    },
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataHashMap, Path},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataHashSet},
        },
        shortcuts::replace_shortcuts_slowly,
        DistanceHeuristic, PathFinding,
    },
    utility::get_progressbar_long_jobs,
};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ContractedGraph {
    upward_graph: VecGraph,
    downward_graph: VecGraph,
    #[serde_as(as = "Vec<(_, _)>")]
    shortcuts: HashMap<(Vertex, Vertex), Vertex>,
    level_to_vertex: Vec<Vertex>,
    vertex_to_level: Vec<Level>,
}

impl PathFinding for ContractedGraph {
    fn shortest_path(&self, source: Vertex, target: Vertex) -> Option<Path> {
        ch_one_to_one_path_wrapped(
            self.upward_graph(),
            self.downward_graph(),
            self.shortcuts(),
            source,
            target,
        )
    }

    fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        ch_one_to_one_wrapped(self.upward_graph(), self.downward_graph(), source, target)
            .map(|(_, distance, _, _)| distance)
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

impl ContractedGraph {
    pub fn by_contraction_with_dijkstra_witness_search<G: Graph + Default + Clone>(
        graph: &ReversibleGraph<G>,
    ) -> ContractedGraph {
        let graph = graph.clone();
        let (level_to_vertex, edges, shortcuts) = contraction_with_witness_search(graph);

        let vertex_to_level = vertex_to_level(&level_to_vertex);

        let mut upward_edges = Vec::new();
        let mut downward_edges = Vec::new();
        for (&(tail, head), &weight) in edges.iter() {
            if vertex_to_level[tail as usize] < vertex_to_level[head as usize] {
                upward_edges.push(WeightedEdge::new(tail, head, weight));
            } else if vertex_to_level[tail as usize] > vertex_to_level[head as usize] {
                downward_edges.push(WeightedEdge::new(head, tail, weight));
            }
        }

        ContractedGraph {
            upward_graph: VecGraph::new(&upward_edges, &level_to_vertex),
            downward_graph: VecGraph::new(&downward_edges, &level_to_vertex),
            shortcuts,
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
    }

    pub fn by_contraction_with_heuristic<G: Graph + Default + Clone>(
        graph: &ReversibleGraph<G>,
        heuristic: &dyn DistanceHeuristic,
    ) -> ContractedGraph {
        let graph = graph.clone();
        let (level_to_vertex, edges, shortcuts) = contraction_with_heuristic(graph, heuristic);

        let vertex_to_level = vertex_to_level(&level_to_vertex);

        let mut upward_edges = Vec::new();
        let mut downward_edges = Vec::new();
        for (&(tail, head), &weight) in edges.iter().progress() {
            if vertex_to_level[tail as usize] < vertex_to_level[head as usize] {
                upward_edges.push(WeightedEdge::new(tail, head, weight));
            } else if vertex_to_level[tail as usize] > vertex_to_level[head as usize] {
                downward_edges.push(WeightedEdge::new(head, tail, weight));
            }
        }

        ContractedGraph {
            upward_graph: VecGraph::new(&upward_edges, &level_to_vertex),
            downward_graph: VecGraph::new(&downward_edges, &level_to_vertex),
            shortcuts,
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
    }

    pub fn by_brute_force<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        level_to_vertex: &Vec<Vertex>,
    ) -> ContractedGraph {
        let vertex_to_level = vertex_to_level(level_to_vertex);

        let (upward_edges, mut shortcuts) = brute_force_contracted_graph_edges(
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

        shortcuts.extend(
            downward_shortcuts
                .into_iter()
                .map(|((tail, head), skiped_vertex)| ((head, tail), skiped_vertex)),
        );

        ContractedGraph {
            upward_graph: VecGraph::new(&upward_edges, &level_to_vertex),
            downward_graph: VecGraph::new(&downward_edges, &level_to_vertex),
            shortcuts,
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
    }

    pub fn upward_graph(&self) -> &dyn Graph {
        &self.upward_graph
    }

    pub fn downward_graph(&self) -> &dyn Graph {
        &self.downward_graph
    }

    pub fn level_to_vertex(&self) -> &Vec<Vertex> {
        &self.level_to_vertex
    }

    pub fn vertex_to_level(&self) -> &Vec<u32> {
        &self.vertex_to_level
    }

    pub fn shortcuts(&self) -> &HashMap<(Vertex, Vertex), Vertex> {
        &self.shortcuts
    }
}

pub fn vertex_to_level(level_to_vertex: &Vec<Vertex>) -> Vec<Level> {
    let mut vertex_to_level = vec![0; level_to_vertex.len()];

    for (level, &vertex) in level_to_vertex.iter().enumerate() {
        vertex_to_level[vertex as usize] = level as u32;
    }

    vertex_to_level
}

pub fn level_to_vertex(vertex_to_level: &Vec<Level>) -> Vec<Vertex> {
    let mut level_to_vertex = vec![0; vertex_to_level.len()];

    for (vertex, &level) in vertex_to_level.iter().enumerate() {
        level_to_vertex[level as usize] = vertex as Vertex;
    }

    level_to_vertex
}

pub fn ch_one_to_one_path_wrapped(
    upward_graph: &dyn Graph,
    downward_graph: &dyn Graph,
    shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
    source: Vertex,
    target: Vertex,
) -> Option<Path> {
    let (vertex, distance, forward_data, backward_data) =
        ch_one_to_one_wrapped(upward_graph, downward_graph, source, target)?;

    let mut vertices = forward_data.get_path(vertex).unwrap().vertices; // (source -> vertex)
    let mut backward_vertices = backward_data.get_path(vertex).unwrap().vertices; // (target -> vertex)

    backward_vertices.reverse(); // (vertex -> target)
    vertices.pop(); // remove double vertex ((source -> vertex) -> (vertex -> target))
    vertices.extend(backward_vertices); // get (source -> target)

    replace_shortcuts_slowly(&mut vertices, shortcuts); // replace the shortcuts

    Some(Path { vertices, distance })
}

pub fn ch_one_to_one_wrapped(
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

    let (vertex, distance) = ch_one_to_one(
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

pub fn ch_one_to_one(
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

    while !forward_queue.is_empty() || !backward_queue.is_empty() {
        ch_search_single_step(
            upward_graph,
            downward_graph,
            forward_data,
            forward_expanded,
            forward_queue,
            backward_data,
            &mut meeting_vertex_and_distance,
        );

        ch_search_single_step(
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

fn ch_search_single_step(
    direction1_graph: &dyn Graph,
    direction2_graph: &dyn Graph,
    direction1_data: &mut dyn DijkstraData,
    direction1_expanded: &mut dyn VertexExpandedData,
    direction1_queue: &mut dyn VertexDistanceQueue,
    direction2_data: &mut dyn DijkstraData,
    meeting_vertex_and_distance: &mut Option<(Vertex, Distance)>,
) {
    if let Some((tail, distance_tail)) = direction1_queue.pop() {
        if direction1_expanded.expand(tail) {
            return;
        }

        // Stall on demand
        for direction2_edge in direction2_graph.edges(tail) {
            if let Some(predecessor_weight) = direction1_data.get_distance(direction2_edge.head) {
                if predecessor_weight + direction2_edge.weight < distance_tail {
                    return;
                }
            }
        }

        // Meeting vertex logic
        if let Some(direction2_distance_tail) = direction2_data.get_distance(tail) {
            let meeting_distance =
                meeting_vertex_and_distance.map_or(Distance::MAX, |(_vertex, distance)| distance);
            let alternative_meeting_distance = distance_tail + direction2_distance_tail;
            if alternative_meeting_distance < meeting_distance {
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

#[cfg(test)]
mod tests {
    use super::{ch_one_to_one_path_wrapped, ContractedGraph};
    use crate::graphs::{large_test_graph, Graph};

    #[test]
    fn contration_by_witness_search() {
        let (graph, tests) = large_test_graph();
        let contracted_graph = ContractedGraph::by_contraction_with_dijkstra_witness_search(&graph);

        for test in tests {
            let path = ch_one_to_one_path_wrapped(
                contracted_graph.upward_graph(),
                contracted_graph.downward_graph(),
                contracted_graph.shortcuts(),
                test.source,
                test.target,
            );

            let distance = path.as_ref().map(|path| path.distance);
            assert_eq!(test.distance, distance);

            let path_distance =
                path.and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
            assert_eq!(test.distance, path_distance)
        }
    }

    #[test]
    fn contration_brute_force() {
        let (graph, tests) = large_test_graph();
        let contracted_graph = ContractedGraph::by_contraction_with_dijkstra_witness_search(&graph);
        let contracted_graph =
            ContractedGraph::by_brute_force(&graph, &contracted_graph.vertex_to_level);

        for test in tests {
            let path = ch_one_to_one_path_wrapped(
                contracted_graph.upward_graph(),
                contracted_graph.downward_graph(),
                contracted_graph.shortcuts(),
                test.source,
                test.target,
            );

            let distance = path.as_ref().map(|path| path.distance);
            assert_eq!(test.distance, distance);

            let path_distance =
                path.and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
            assert_eq!(test.distance, path_distance)
        }
    }
}
