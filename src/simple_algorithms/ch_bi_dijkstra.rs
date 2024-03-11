use std::collections::BinaryHeap;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use crate::{
    ch::preprocessor::ContractedGraph,
    dijkstra_data::DijkstraData,
    graphs::{
        edge::DirectedEdge,
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
    queue::heap_queue::State,
};

#[derive(Clone)]
pub struct ChDijkstra {
    pub graph: FastGraph,
    pub shortcuts: HashMap<DirectedEdge, u32>,
    pub levels: Vec<Vec<VertexId>>,
}

impl PathFinding for ChDijkstra {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        self.get_data(&route_request)
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl ChDijkstra {
    pub fn new(contracted_grap: &ContractedGraph) -> ChDijkstra {
        let shortcuts = contracted_grap.shortcuts_map.iter().cloned().collect();
        let graph = FastGraph::from_graph(&contracted_grap.graph);
        ChDijkstra {
            graph,
            shortcuts,
            levels: contracted_grap.levels.clone(),
        }
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
