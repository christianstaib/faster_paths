use crate::{
    graphs::{vec_vec_graph::VecVecGraph, Distance, Graph, Vertex},
    search::{collections::dijkstra_data::DijkstraData, dijkstra::dijkstra_one_to_all_wraped},
};

pub struct ContractedGraph {
    pub up_graph: VecVecGraph,
    pub down_graph: VecVecGraph,
    pub level_to_vertex: Vec<Vertex>,
}

impl ContractedGraph {
    pub fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        let up_weights = dijkstra_one_to_all_wraped(&self.up_graph, source);
        let down_weights = dijkstra_one_to_all_wraped(&self.down_graph, target);

        let mut min_distance = Distance::MAX;
        for vertex in 0..std::cmp::max(
            self.up_graph.number_of_vertices(),
            self.down_graph.number_of_vertices(),
        ) {
            let alt_distance = match (
                up_weights.get_distance(vertex),
                down_weights.get_distance(vertex),
            ) {
                (Some(a), Some(b)) => a + b,
                _ => Distance::MAX,
            };

            if alt_distance < min_distance {
                min_distance = alt_distance;
            }
        }

        if min_distance == Distance::MAX {
            return None;
        }

        Some(min_distance)
    }
}
