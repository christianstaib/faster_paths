use crate::{
    dijkstra_data::{dijkstra_data_vec::DijkstraDataVec, DijkstraData},
    graphs::{
        graph::Graph,
        path::{Path, PathFinding, ShortestPathRequest},
        VertexId, Weight,
    },
    queue::DijkstraQueueElement,
};

/// Works with slower Graph struct, not faster FastGrap
#[derive(Clone)]
pub struct SlowDijkstra {
    graph: Graph,
}

impl PathFinding for SlowDijkstra {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let data = self.get_data(route_request.source(), route_request.target());
        data.get_path(route_request.target())
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl SlowDijkstra {
    pub fn new(graph: Graph) -> SlowDijkstra {
        SlowDijkstra { graph }
    }

    pub fn get_data(&self, source: VertexId, target: VertexId) -> DijkstraDataVec {
        let mut data = DijkstraDataVec::new(self.graph.number_of_vertices() as usize, source);

        while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
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
