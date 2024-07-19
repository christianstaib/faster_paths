use self::edge::{Edge, WeightedEdge};

pub mod adjacency_list_graph;
pub mod edge;
pub mod graph_factory;
pub mod graph_functions;
pub mod path;
pub mod reversible_graph;
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
    ) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_>;

    fn in_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_>;

    fn get_edge_weight(&self, edge: &Edge) -> Option<Weight> {
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
    fn set_edge(&mut self, edge: &WeightedEdge);

    // set OR updates eges. may be faster than update edges
    fn set_edges(&mut self, edges: &[WeightedEdge]) {
        for edge in edges {
            self.set_edge(edge);
        }
    }

    fn remove_vertex(&mut self, vertex: VertexId);
}
