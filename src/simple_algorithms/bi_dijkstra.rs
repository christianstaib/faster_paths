use crate::{
    dijkstra_data::DijkstraData,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        Weight,
    },
    queue::DijkstraQueueElement,
};

use super::bidirectional_helpers::path_from_bidirectional_search;

#[derive(Clone)]
pub struct BiDijkstra<'a> {
    pub graph: &'a FastGraph,
}

impl<'a> PathFinding for BiDijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        self.get_data(route_request)
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl<'a> BiDijkstra<'a> {
    pub fn new(graph: &'a FastGraph) -> BiDijkstra {
        BiDijkstra { graph }
    }

    pub fn get_data(&self, request: &ShortestPathRequest) -> Option<Path> {
        let mut forward_data =
            DijkstraData::new(self.graph.number_of_vertices() as usize, request.source());
        let mut backward_data =
            DijkstraData::new(self.graph.number_of_vertices() as usize, request.target());

        self.get_route_data(&mut forward_data, &mut backward_data)
    }

    pub fn get_route_data(
        &self,
        forward: &mut DijkstraData,
        backward: &mut DijkstraData,
    ) -> Option<Path> {
        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        let mut forward_max_weight = 0;
        let mut backward_max_weight = 0;

        while !forward.is_empty() || !backward.is_empty() {
            if let Some(DijkstraQueueElement { vertex, .. }) = forward.pop() {
                let forward_weight = forward.vertices[vertex as usize].weight.unwrap();
                if forward_weight > forward_max_weight {
                    forward_max_weight = forward_weight;
                }
                if let Some(backward_weight) = backward.vertices[vertex as usize].weight {
                    let cost = forward_weight + backward_weight;
                    if cost < meeting_weight {
                        meeting_weight = cost;
                        meeting_vertex = vertex;
                    }
                }
                self.graph
                    .out_edges(vertex)
                    .iter()
                    .for_each(|edge| forward.update(vertex, edge.head, edge.weight));
            }

            if let Some(DijkstraQueueElement { vertex, .. }) = backward.pop() {
                if forward.vertices[vertex as usize].is_expanded {
                    let backward_weight = backward.vertices[vertex as usize].weight.unwrap();
                    if backward_weight > backward_max_weight {
                        backward_max_weight = backward_weight;
                    }
                    if let Some(forward_weight) = forward.vertices[vertex as usize].weight {
                        let cost = forward_weight + backward_weight;
                        if cost < meeting_weight {
                            meeting_weight = cost;
                            meeting_vertex = vertex;
                        }
                    }
                }
                self.graph.in_edges(vertex).iter().for_each(|edge| {
                    backward.update(vertex, edge.tail, edge.weight);
                });
            }

            if forward_max_weight + backward_max_weight >= meeting_weight {
                break;
            }
        }

        path_from_bidirectional_search(meeting_vertex, forward, backward)
    }
}
