use serde_derive::{Deserialize, Serialize};

use super::{
    dijkstra_data::DijkstraData,
    types::{VertexId, Weight},
};

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

#[derive(Clone)]
pub struct RouteResponse {
    pub route: Option<Path>,
    pub data: Vec<DijkstraData>,
}

pub trait Routing {
    fn get_route(&self, route_request: &PathRequest) -> RouteResponse;
}

impl RouteResponse {
    pub fn get_cost(&self) -> Option<u32> {
        let mut cost = None;
        if let Some(route) = &self.route {
            cost = Some(route.weight);
        }
        cost
    }
}

impl PathRequest {
    pub fn reversed(&self) -> PathRequest {
        PathRequest {
            source: self.target,
            target: self.source,
        }
    }
}
