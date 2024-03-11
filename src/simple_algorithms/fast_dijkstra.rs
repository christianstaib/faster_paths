use std::{collections::BinaryHeap, usize};

use crate::{
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
    queue::heap_queue::State,
};

#[derive(Clone)]
pub struct FastDijkstra<'a> {
    graph: &'a FastGraph,
}

impl<'a> PathFinding for FastDijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (weight, predecessor) = self.get_data(route_request.source(), route_request.target());

        let mut vertices = vec![route_request.target()];
        let mut current = route_request.target();
        while let Some(predecessor) = predecessor.get(current as usize)? {
            current = *predecessor;
            vertices.push(current);
        }
        vertices.reverse();
        Some(Path {
            weight: *weight.get(route_request.target() as usize)?,
            vertices,
        })
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl<'a> FastDijkstra<'a> {
    pub fn new(graph: &'a FastGraph) -> FastDijkstra {
        FastDijkstra { graph }
    }

    pub fn get_data(
        &self,
        source: VertexId,
        target: VertexId,
    ) -> (Vec<u32>, Vec<Option<VertexId>>) {
        let mut queue = BinaryHeap::new();
        let mut weight = vec![u32::MAX; self.graph.number_of_vertices() as usize];
        let mut predcessor = vec![None; self.graph.number_of_vertices() as usize];

        queue.push(State::new(0, source));
        weight[source as usize] = 0;

        while let Some(State { vertex, .. }) = queue.pop() {
            if vertex == target {
                break;
            }

            for edge in self.graph.out_edges(vertex).iter() {
                let alternative_weight = weight[vertex as usize] + edge.weight;
                let current_weight = weight[edge.head as usize];
                if alternative_weight < current_weight {
                    queue.push(State::new(alternative_weight, edge.head));
                    weight[edge.head as usize] = alternative_weight;
                    predcessor[edge.head as usize] = Some(vertex);
                }
            }
        }

        (weight, predcessor)
    }
}
