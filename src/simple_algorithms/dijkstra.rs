use crate::{
    dijkstra_data::DijkstraData,
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
    queue::heap_queue::State,
};

#[derive(Clone)]
pub struct Dijkstra<'a> {
    graph: &'a FastGraph,
}

impl<'a> PathFinding for Dijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let data = self.get_data(route_request.source(), route_request.target());
        data.get_path(route_request.target())
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl<'a> Dijkstra<'a> {
    pub fn new(graph: &'a FastGraph) -> Dijkstra {
        Dijkstra { graph }
    }

    pub fn get_data(&self, source: VertexId, target: VertexId) -> DijkstraData {
        let mut data = DijkstraData::new(self.graph.number_of_vertices() as usize, source);

        while let Some(State { vertex, .. }) = data.pop() {
            if vertex == target {
                return data;
            }
            self.graph
                .out_edges(vertex)
                .iter()
                .for_each(|edge| data.update(vertex, edge.head, edge.weight));
        }

        data
    }
}
