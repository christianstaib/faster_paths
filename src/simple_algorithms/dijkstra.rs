use crate::{
    dijkstra_data::{dijkstra_data_vec::DijkstraDataVec, DijkstraData},
    graphs::{
        path::{Path, PathFinding, ShortestPathRequest},
        Graph, VertexId, Weight,
    },
    queue::DijkstraQueueElement,
};

#[derive(Clone)]
pub struct Dijkstra<'a> {
    graph: &'a dyn Graph,
}

impl<'a> PathFinding for Dijkstra<'a> {
    fn shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let data = self.get_data(route_request.source(), route_request.target());
        data.get_path(route_request.target())
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl<'a> Dijkstra<'a> {
    pub fn new(graph: &dyn Graph) -> Dijkstra {
        Dijkstra { graph }
    }

    pub fn get_data(&self, source: VertexId, target: VertexId) -> DijkstraDataVec {
        let mut data = DijkstraDataVec::new(self.graph.number_of_vertices() as usize, source);

        while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
            if vertex == target {
                return data;
            }
            self.graph
                .out_edges(vertex)
                .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
        }

        data
    }

    pub fn single_source(&self, source: VertexId) -> DijkstraDataVec {
        let mut data = DijkstraDataVec::new(self.graph.number_of_vertices() as usize, source);

        while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
            self.graph
                .out_edges(vertex)
                .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
        }

        data
    }

    pub fn single_source_dijkstra_rank(
        &self,
        source: VertexId,
    ) -> (Vec<Option<u32>>, DijkstraDataVec) {
        let mut data = DijkstraDataVec::new(self.graph.number_of_vertices() as usize, source);
        let mut dijkstra_rank = vec![None; self.graph.number_of_vertices() as usize];

        let mut current_dijkstra_rank = 0;
        while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
            current_dijkstra_rank += 1;
            dijkstra_rank[vertex as usize] = Some(current_dijkstra_rank);
            self.graph
                .out_edges(vertex)
                .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
        }

        (dijkstra_rank, data)
    }

    pub fn single_target(&self, target: VertexId) -> DijkstraDataVec {
        let mut data = DijkstraDataVec::new(self.graph.number_of_vertices() as usize, target);

        while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
            self.graph
                .in_edges(vertex)
                .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
        }

        data
    }
}
