//! Collection of datastructures that represent graphs.
//!

mod adjacency_list_graph;
mod csr_graph;
mod directional_adjacency_list_graph;
mod edge;
mod edge_like;
mod graph_like;
mod topological_sorting;

pub use adjacency_list_graph::AdjacencyListGraph;
pub use csr_graph::CsrGraph;
pub use directional_adjacency_list_graph::DirectionalAdjacencyListGraph;
pub use edge::Edge;
pub use edge::WeightedEdge;
pub use edge_like::EdgeLike;
pub use graph_like::GraphLike;
pub use topological_sorting::compute_topological_layers;
