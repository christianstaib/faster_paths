use crate::{
    dijkstra_data::DijkstraData,
    fast_graph::FastGraph,
    path::{PathRequest, PathRequestResponse, Pathfinding},
};

#[derive(Clone)]
pub struct Dijkstra<'a> {
    graph: &'a FastGraph,
}

impl<'a> Pathfinding for Dijkstra<'a> {
    fn get_path(&self, route_request: &PathRequest) -> PathRequestResponse {
        let data = self.get_forward_data(route_request.source);
        let route = data.get_route(route_request.target);
        PathRequestResponse {
            route,
            data: vec![data],
        }
    }
}

impl<'a> Dijkstra<'a> {
    pub fn new(graph: &'a FastGraph) -> Dijkstra {
        Dijkstra { graph }
    }

    pub fn get_forward_data(&self, source: u32) -> DijkstraData {
        let mut data = DijkstraData::new(self.graph.num_nodes() as usize, source);

        while let Some(state) = data.pop() {
            self.graph
                .out_edges(state.value)
                .iter()
                .for_each(|edge| data.update(state.value, edge.head, edge.cost));
        }

        data
    }
}
