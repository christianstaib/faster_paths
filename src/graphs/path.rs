use serde_derive::{Deserialize, Serialize};

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
pub struct ShortestPathValidation {
    pub request: ShortestPathRequest,
    pub weight: Option<Weight>,
}

/// Defines the behavior for routing algorithms.
///
/// This trait defines a method `get_path` that must be implemented by any struct that
/// performs routing in a graph, allowing for the retrieval of paths based on given shortest path requests.
pub trait PathFinding {
    /// Retrieves the shortest path between two vertices in a graph as specified by a path request.
    ///
    /// # Arguments
    /// * `path_request` - A reference to a `ShortestPathRequest` specifying the source and target vertices
    ///                    for which the shortest path needs to be found.
    ///
    /// # Returns
    /// * `Option<Path>` - Returns `Some(Path)` if a shortest path exists between the source and target vertices,
    ///                    otherwise returns `None`. The `Path` struct encapsulates the sequence of vertices
    ///                    forming the shortest path and its total weight.
    fn get_shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path>;

    /// Retrieves the weight of the shortest path between two vertices in a graph as specified by a path request.
    ///
    /// This method is designed to calculate and return the total weight of the shortest path between the source
    /// and target vertices defined in the `ShortestPathRequest`. It is particularly useful for applications where
    /// the weight of the path is the primary concern, rather than the path itself. It
    /// might be faster than `get_shortest_path` as no path needs to be constructed.
    ///
    /// # Arguments
    /// * `path_request` - A reference to a `ShortestPathRequest` that specifies the source and target vertices
    ///                    for which the weight of the shortest path is to be determined.
    ///
    /// # Returns
    /// * `Option<Weight>` - Returns `Some(Weight)` if a shortest path exists between the source and target vertices,
    ///                      indicating the total weight of that path. Returns `None` if no path exists or if the path
    ///                      cannot be calculated. This could be due to various reasons such as disconnected vertices
    ///                      or an overflow in calculating the weight.
    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight>;
}
