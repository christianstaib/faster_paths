use crate::{
    dijkstra_data::DijkstraData,
    graphs::{
        graph::Graph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::Weight,
    },
};

#[derive(Clone)]
pub struct SlowDijkstra<'a> {
    graph: &'a Graph,
}

impl<'a> PathFinding for SlowDijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let data = self.get_data(route_request.source());
        data.get_path(route_request.target())
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl<'a> SlowDijkstra<'a> {
    pub fn new(graph: &'a Graph) -> SlowDijkstra {
        SlowDijkstra { graph }
    }

    pub fn get_data(&self, source: u32) -> DijkstraData {
        let mut data = DijkstraData::new(self.graph.number_of_vertices() as usize, source);

        while let Some(state) = data.pop() {
            self.graph
                .out_edges(state.vertex)
                .iter()
                .for_each(|edge| data.update(state.vertex, edge.head, edge.weight));
        }

        data
    }
}
