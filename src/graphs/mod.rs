use self::{
    edge::{DirectedEdge, DirectedWeightedEdge},
    path::{PathFinding, ShortestPathRequest},
};
use crate::classical_search::dijkstra::Dijkstra;

pub mod adjacency_vec_graph;
pub mod edge;
pub mod graph_factory;
pub mod graph_functions;
pub mod hash_graph;
pub mod matrix_graph;
pub mod path;
pub mod reversible_hash_graph;
pub mod reversible_vec_graph;
pub mod vec_graph;

pub type VertexId = u32;
pub type EdgeId = u32;
pub type Weight = u32;

pub trait Graph: Send + Sync {
    fn out_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_>;

    fn in_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_>;

    fn get_edge_weight(&self, edge: &DirectedEdge) -> Option<Weight> {
        Some(
            self.out_edges(edge.tail())
                .find(|out_edge| out_edge.head() == edge.head())?
                .weight(),
        )
    }

    fn number_of_vertices(&self) -> u32;

    fn number_of_edges(&self) -> u32;

    // insert edge if not pressent or updated edge weight if new edge weight is
    // smaller than currents.
    fn set_edge(&mut self, edge: &DirectedWeightedEdge);

    // set OR updates eges. may be faster than update edges
    fn set_edges(&mut self, edges: &[DirectedWeightedEdge]) {
        for edge in edges {
            self.set_edge(edge);
        }
    }

    fn remove_vertex(&mut self, vertex: VertexId);
}

impl PathFinding for dyn Graph {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<path::Path> {
        let dijkstra = Dijkstra::new(self);
        dijkstra.shortest_path(path_request)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let dijkstra = Dijkstra::new(self);
        dijkstra.shortest_path_weight(path_request)
    }

    fn number_of_vertices(&self) -> u32 {
        todo!()
    }
}
