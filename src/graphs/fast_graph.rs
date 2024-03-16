use serde::{Deserialize, Serialize};

use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge},
    fast_edge_access::{FastInEdgeAccess, FastOutEdgeAccess},
    graph::Graph,
    VertexId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct FastGraph {
    number_of_vertices: u32,
    out_edges: FastOutEdgeAccess,
    in_edges: FastInEdgeAccess,
}

impl FastGraph {
    pub fn from_graph(graph: &Graph) -> FastGraph {
        let number_of_vertices = graph.number_of_vertices();
        let out_edges: Vec<_> = (0..number_of_vertices)
            .map(|tail| graph.out_edges(tail).clone())
            .collect();
        let in_edges: Vec<_> = (0..number_of_vertices)
            .map(|tail| graph.in_edges(tail).clone())
            .collect();
        let out_edges = FastOutEdgeAccess::new(&out_edges);
        let in_edges = FastInEdgeAccess::new(&in_edges);

        FastGraph {
            number_of_vertices,
            out_edges,
            in_edges,
        }
    }

    pub fn number_of_vertices(&self) -> u32 {
        self.number_of_vertices
    }

    pub fn out_edges(&self, source: VertexId) -> &[DirectedTaillessWeightedEdge] {
        self.out_edges.edges(source)
    }

    pub fn in_edges(&self, target: VertexId) -> &[DirectedHeadlessWeightedEdge] {
        self.in_edges.edges(target)
    }

    pub fn max_edge_weight(&self) -> Option<u32> {
        let max_out_weight = self.out_edges.max_edge_weight();
        let max_in_weight = self.in_edges.max_edge_weight();
        match (max_out_weight, max_in_weight) {
            (None, None) => None,
            (None, Some(max_in_weight)) => Some(max_in_weight),
            (Some(max_out_weight), None) => Some(max_out_weight),
            (Some(max_out_weight), Some(max_in_weight)) => {
                Some(std::cmp::max(max_out_weight, max_in_weight))
            }
        }
    }
}
