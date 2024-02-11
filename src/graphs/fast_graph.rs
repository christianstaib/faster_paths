use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge},
    fast_edge_access::{FastInEdgeAccess, FastOutEdgeAccess},
    graph::Graph,
    types::VertexId,
};

#[derive(Clone)]
pub struct FastGraph {
    num_nodes: u32,
    out_edges: FastOutEdgeAccess,
    in_edges: FastInEdgeAccess,
}

impl FastGraph {
    pub fn from_graph(graph: &Graph) -> FastGraph {
        let num_nodes = graph.in_edges.len() as u32;
        let out_edges = FastOutEdgeAccess::new(&graph.out_edges);
        let in_edges = FastInEdgeAccess::new(&graph.in_edges);

        FastGraph {
            num_nodes,
            out_edges,
            in_edges,
        }
    }

    pub fn num_nodes(&self) -> u32 {
        self.num_nodes
    }

    pub fn out_edges(&self, source: VertexId) -> &[DirectedTaillessWeightedEdge] {
        self.out_edges.edges(source)
    }

    pub fn in_edges(&self, target: VertexId) -> &[DirectedHeadlessWeightedEdge] {
        self.in_edges.edges(target)
    }
}
