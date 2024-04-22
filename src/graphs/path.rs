use serde::{Deserialize, Serialize};

use super::{VertexId, Weight};

/// Represents a request for finding a shortest path in a graph.
///
/// This struct is used to encapsulate the information required to find a path
/// from a source vertex to a target vertex in a graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathRequest {
    source: VertexId,
    target: VertexId,
}

impl ShortestPathRequest {
    pub fn new(source: VertexId, target: VertexId) -> Option<ShortestPathRequest> {
        if source == target {
            return None;
        }

        Some(ShortestPathRequest { source, target })
    }

    pub fn source(&self) -> VertexId {
        self.source
    }

    pub fn target(&self) -> VertexId {
        self.target
    }
}

/// Represents a path in a graph.
///
/// This struct encapsulates the vertices that form a path in the graph and the total weight
/// associated with traversing this path.
#[derive(Clone)]
pub struct Path {
    pub vertices: Vec<VertexId>,
    pub weight: Weight,
}

/// Represents a request for validating a shortest path in a graph.
///
/// This struct is used to encapsulate a shortest path request along with the weight of a shortest
/// path, if there exists one.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathTestCase {
    pub request: ShortestPathRequest,
    pub weight: Option<Weight>,
    pub dijkstra_rank: u32,
}

pub trait PathFinding: Send + Sync {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path>;

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight>;
}

pub trait PathFindingWithInternalState {
    fn shortest_path(&mut self, path_request: &ShortestPathRequest) -> Option<Path>;

    fn shortest_path_weight(&mut self, path_request: &ShortestPathRequest) -> Option<Weight>;
}
