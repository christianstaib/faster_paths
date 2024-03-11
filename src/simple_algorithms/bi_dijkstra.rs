use crate::{
    dijkstra_data::DijkstraData,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
};

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
        let mut forward_data = DijkstraData::new(self.graph.num_nodes() as usize, request.source());
        let mut backward_data =
            DijkstraData::new(self.graph.num_nodes() as usize, request.target());

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
                if let Some(backward_cost) = backward.verticies[state.value as usize].weight {
                    let forward_cost = forward.verticies[state.value as usize].weight.unwrap();
                    let cost = forward_cost + backward_cost;
                    if cost < minimal_cost {
                        minimal_cost = cost;
                        minimal_cost_vertex = state.value;
                    }
                }
                self.graph
                    .out_edges(state.value)
                    .iter()
                    .for_each(|edge| forward.update(state.value, edge.head, edge.weight));
            }

            if let Some(state) = backward.pop() {
                if forward.verticies[state.value as usize].is_expanded {
                    if let Some(forward_cost) = forward.verticies[state.value as usize].weight {
                        let backward_cost =
                            backward.verticies[state.value as usize].weight.unwrap();
                        let cost = forward_cost + backward_cost;
                        if cost < minimal_cost {
                            minimal_cost = cost;
                            minimal_cost_vertex = state.value;
                        }
                    }
                }
                self.graph.in_edges(state.value).iter().for_each(|edge| {
                    backward.update(state.value, edge.tail, edge.weight);
                });
            }
        }

        construct_route(minimal_cost_vertex, forward, backward)
    }
}

fn construct_route(
    contact_node: VertexId,
    forward_data: &DijkstraData,
    backward_data: &DijkstraData,
) -> Option<Path> {
    let mut forward_route = forward_data.get_route(contact_node)?;
    let mut backward_route = backward_data.get_route(contact_node)?;
    backward_route.vertices.pop();
    backward_route.vertices.reverse();
    forward_route.vertices.extend(backward_route.vertices);
    forward_route.weight += backward_route.weight;

    Some(forward_route)
}
