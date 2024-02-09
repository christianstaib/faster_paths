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

impl RouteValidationRequest {
    pub fn from_str(str: &str) -> Option<RouteValidationRequest> {
        let line: Vec<_> = str.split(',').collect();
        let mut cost = None;
        if let Ok(str_cost) = line[2].parse::<u32>() {
            cost = Some(str_cost);
        }
        Some(RouteValidationRequest {
            request: PathRequest {
                source: line[0].parse().ok()?,
                target: line[1].parse().ok()?,
            },
            cost,
        })
    }
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
