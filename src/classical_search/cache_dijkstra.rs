use std::usize;

use ahash::{HashMap, HashMapExt};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::{
    classical_search::dijkstra::Dijkstra,
    dijkstra_data::{
        dijkstra_data_vec::{DijkstraDataVec, DijsktraEntry},
        DijkstraData,
    },
    graphs::{
        graph_functions::{all_edges, hitting_set, random_paths},
        path::PathFinding,
        reversible_vec_graph::ReversibleVecGraph,
        Graph, VertexId,
    },
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

    pub fn with_pathfinder(
        graph: &'a dyn Graph,
        number_of_random_pairs: u32,
        path_finder: &dyn PathFinding,
    ) -> CacheDijkstra<'a> {
        println!("Generating {} random paths", number_of_random_pairs);
        let graph_copy = ReversibleVecGraph::from_edges(&all_edges(graph));
        let dijkstra = Dijkstra {
            graph: Box::new(graph_copy),
        };
        let paths = random_paths(
            &dijkstra,
            number_of_random_pairs,
            graph.number_of_vertices(),
        );

        println!("generating hitting set");
        let (hitting_setx, _) = hitting_set(&paths, graph.number_of_vertices());

        println!("generating random pair test");

        println!("generating cache");
        let mut cache_dijkstra = CacheDijkstra::new(graph);
        cache_dijkstra.cache = hitting_setx
            .par_iter()
            .progress()
            .map(|&vertex| {
                let data = cache_dijkstra.single_source(vertex);
                let data = data.vertices;
                (vertex, data)
            })
            .collect();

        cache_dijkstra
    }

    pub fn single_source(&self, source: VertexId) -> DijkstraDataVec {
        let mut data = DijkstraDataVec::new(self.graph.number_of_vertices() as usize, source);

        while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
            if let Some(cached_data) = self.cache.get(&vertex) {
                let vertex_weight = data.vertices.get(vertex as usize).unwrap().weight.unwrap();

                for vertex in 0..self.graph.number_of_vertices() {
                    let cached_entry = cached_data.get(vertex as usize).unwrap();
                    let this_entry = data.vertices.get_mut(vertex as usize).unwrap();

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
}
