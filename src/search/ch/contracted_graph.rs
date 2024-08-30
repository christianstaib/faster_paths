use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::{
    brute_force::brute_force_contracted_graph_edges,
    search::{one_to_one_wrapped_distance, one_to_one_wrapped_path},
};
use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_graph::VecGraph, Distance, Graph, Level, Vertex,
        WeightedEdge,
    },
    search::{collections::dijkstra_data::Path, PathFinding},
    utility::get_progressbar_long_jobs,
};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ContractedGraph {
    upward_graph: VecGraph,
    downward_graph: VecGraph,
    shortcuts: HashMap<(Vertex, Vertex), Vertex>,
    level_to_vertex: Vec<Vertex>,
    vertex_to_level: Vec<Level>,
}

impl ContractedGraph {
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

pub fn generate_contracted_graph_edge_vec(
    level_to_vertex: Vec<u32>,
    edges: Vec<WeightedEdge>,
    shortcuts: HashMap<(u32, u32), u32>,
) -> ContractedGraph {
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let mut upward_edges = Vec::new();
    let mut downward_edges = Vec::new();
    for edge in edges.into_iter() {
        if vertex_to_level[edge.tail as usize] < vertex_to_level[edge.head as usize] {
            upward_edges.push(edge);
        } else if vertex_to_level[edge.tail as usize] > vertex_to_level[edge.head as usize] {
            downward_edges.push(edge);
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

#[cfg(test)]
mod tests {
    use super::ContractedGraph;
    use crate::{
        graphs::{large_test_graph, Graph},
        search::ch::search::one_to_one_wrapped_path,
    };

    #[test]
    fn contration_by_witness_search() {
        let (graph, tests) = large_test_graph();
        let contracted_graph = ContractedGraph::with_dijkstra_witness_search(&graph, u32::MAX);

        for test in tests {
            let path = one_to_one_wrapped_path(
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
        let contracted_graph = ContractedGraph::with_dijkstra_witness_search(&graph, u32::MAX);
        let contracted_graph =
            ContractedGraph::by_brute_force(&graph, &contracted_graph.vertex_to_level);

        for test in tests {
            let path = one_to_one_wrapped_path(
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
