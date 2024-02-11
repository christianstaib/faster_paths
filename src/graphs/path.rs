use serde_derive::{Deserialize, Serialize};

use super::types::{VertexId, Weight};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathRequest {
    pub source: VertexId,
    pub target: VertexId,
}

#[derive(Clone)]
pub struct Path {
    pub vertices: Vec<VertexId>,
    pub weight: Weight,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteValidationRequest {
    pub request: PathRequest,
    pub cost: Option<u32>,
}

pub trait Routing {
    fn get_path(&self, route_request: &PathRequest) -> Option<Path>;
}

impl PathRequest {
    pub fn reversed(&self) -> PathRequest {
        PathRequest {
            source: self.target,
            target: self.source,
        }
    }
}
