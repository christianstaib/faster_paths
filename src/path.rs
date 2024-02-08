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
pub struct PathValidationRequest {
    pub request: PathRequest,
    pub weight: Option<u32>,
}

#[derive(Clone)]
pub struct PathRequestResponse {
    pub route: Option<Path>,
    pub data: Vec<DijkstraData>,
}

pub trait Pathfinding {
    fn get_path(&self, route_request: &PathRequest) -> PathRequestResponse;
}

impl PathRequestResponse {
    pub fn get_weight(&self) -> Option<Weight> {
        let mut cost = None;
        if let Some(route) = &self.route {
            cost = Some(route.weight);
        }
        cost
    }
}
