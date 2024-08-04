use serde::{Deserialize, Serialize};

use crate::graphs::{VertexId, Weight};

/// Represents a request for validating a shortest path in a graph.
///
/// This struct is used to encapsulate a shortest path request along with the
/// weight of a shortest path, if there exists one.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathTestCase {
    pub request: ShortestPathRequest,
    pub weight: Option<Weight>,
    pub dijkstra_rank: u32,
}

/// Represents a request for finding a shortest path in a graph.
///
/// This struct is used to encapsulate the information required to find a path
/// from a source vertex to a target vertex in a graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathRequest {
    pub source: VertexId,
    pub target: VertexId,
}
