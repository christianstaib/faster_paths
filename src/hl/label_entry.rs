use serde::{Deserialize, Serialize};

use crate::graphs::{VertexId, Weight};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub vertex: VertexId,
    pub predecessor: Option<u32>,
    pub weight: Weight,
}

impl LabelEntry {
    pub fn new(vertex: VertexId) -> LabelEntry {
        LabelEntry {
            vertex,
            predecessor: None,
            weight: 0,
        }
    }
}
