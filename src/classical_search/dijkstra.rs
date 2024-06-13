use std::usize;

use itertools::sorted;

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
    pub graph: &'a dyn Graph,
}

impl<'a> PathFinding for Dijkstra<'a> {
    fn shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let data = get_data(self.graph, route_request.source(), route_request.target());
        data.get_path(route_request.target())
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.shortest_path(path_request)?;
        Some(data.weight)
    }

    fn number_of_vertices(&self) -> u32 {
        self.graph.number_of_vertices()
    }
}

pub fn get_data(graph: &dyn Graph, source: VertexId, target: VertexId) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        if vertex == target {
            return data;
        }
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    data
}

pub fn single_source(graph: &dyn Graph, source: VertexId) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    data
}

pub fn single_source_with_order(
    graph: &dyn Graph,
    source: VertexId,
    order: &[u32],
) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        if order[vertex as usize] <= order[source as usize] {
            continue;
        }
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    data
}

pub fn single_source_dijkstra_rank(
    graph: &dyn Graph,
    source: VertexId,
) -> (Vec<Option<u32>>, DijkstraDataVec) {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);
    let mut dijkstra_rank = vec![None; graph.number_of_vertices() as usize];

    let mut current_dijkstra_rank = 0;
    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        current_dijkstra_rank += 1;
        dijkstra_rank[vertex as usize] = Some(current_dijkstra_rank);
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    (dijkstra_rank, data)
}

pub fn single_target(graph: &dyn Graph, target: VertexId) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, target);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        graph
            .in_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.tail(), edge.weight()));
    }

    data
}
