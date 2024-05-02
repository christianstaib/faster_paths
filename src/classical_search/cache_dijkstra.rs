use std::usize;

use ahash::{HashMap, HashMapExt};

use crate::{
    dijkstra_data::{
        dijkstra_data_vec::{DijkstraDataVec, DijsktraEntry},
        DijkstraData,
    },
    graphs::{Graph, VertexId},
    queue::DijkstraQueueElement,
};

#[derive(Clone)]
pub struct CacheDijkstra<'a> {
    pub cache: HashMap<VertexId, Vec<DijsktraEntry>>,
    graph: &'a dyn Graph,
}

impl<'a> CacheDijkstra<'a> {
    pub fn new(graph: &dyn Graph) -> CacheDijkstra {
        let cache = HashMap::new();
        CacheDijkstra { cache, graph }
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
            if let Some(cached_data) = self.cache.get(&vertex) {
                let vertex_weight = data.vertices.get(vertex as usize).unwrap().weight.unwrap();

                for i in 0..cached_data.len() {
                    let cached_entry = cached_data.get(i).unwrap();
                    let this_entry = data.vertices.get_mut(i).unwrap();

                    if let Some(cached_weight) = cached_entry.weight {
                        let cached_weight = cached_weight + vertex_weight;
                        if cached_weight < this_entry.weight.unwrap_or(u32::MAX) {
                            this_entry.weight = Some(cached_weight);
                            this_entry.predecessor = cached_entry.predecessor;
                        }
                    }
                }
            }

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
                .for_each(|edge| data.update(vertex, edge.tail(), edge.weight()));
        }

        data
    }
}