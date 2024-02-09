use crate::{
    dijkstra_data::DijkstraData,
    fast_graph::FastGraph,
    path::{Path, PathRequest, Routing},
};

#[derive(Clone)]
pub struct Dijkstra<'a> {
    graph: &'a FastGraph,
}

impl<'a> Routing for Dijkstra<'a> {
    fn get_path(&self, route_request: &PathRequest) -> Option<Path> {
        let data = self.get_forward_data(route_request.source);
        data.get_route(route_request.target)
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
