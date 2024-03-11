use crate::{
    dijkstra_data::DijkstraData,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::Weight,
    },
};

use super::bidirectional_helpers::path_from_bidirectional_search;

#[derive(Clone)]
pub struct BiDijkstra<'a> {
    pub graph: &'a FastGraph,
}

impl<'a> PathFinding for BiDijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        self.get_data(&route_request)
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

        let route = self.get_route_data(&mut forward_data, &mut backward_data);

        route
    }

    pub fn get_route_data(
        &self,
        forward: &mut DijkstraData,
        backward: &mut DijkstraData,
    ) -> Option<Path> {
        let mut minimal_cost = u32::MAX;
        let mut minimal_cost_vertex = u32::MAX;

        while !forward.is_empty() || !backward.is_empty() {
            if let Some(state) = forward.pop() {
                if let Some(backward_cost) = backward.verticies[state.vertex as usize].weight {
                    let forward_cost = forward.verticies[state.vertex as usize].weight.unwrap();
                    let cost = forward_cost + backward_cost;
                    if cost < minimal_cost {
                        minimal_cost = cost;
                        minimal_cost_vertex = state.vertex;
                    }
                }
                self.graph
                    .out_edges(state.vertex)
                    .iter()
                    .for_each(|edge| forward.update(state.vertex, edge.head, edge.weight));
            }

            if let Some(state) = backward.pop() {
                if forward.verticies[state.vertex as usize].is_expanded {
                    if let Some(forward_cost) = forward.verticies[state.vertex as usize].weight {
                        let backward_cost =
                            backward.verticies[state.vertex as usize].weight.unwrap();
                        let cost = forward_cost + backward_cost;
                        if cost < minimal_cost {
                            minimal_cost = cost;
                            minimal_cost_vertex = state.vertex;
                        }
                    }
                }
                self.graph.in_edges(state.vertex).iter().for_each(|edge| {
                    backward.update(state.vertex, edge.tail, edge.weight);
                });
            }
        }

        path_from_bidirectional_search(minimal_cost_vertex, forward, backward)
    }
}
