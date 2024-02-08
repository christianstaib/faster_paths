use serde_derive::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::types::{VertexId, Weight};

#[derive(Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub vertex: VertexId,
    pub predecessor: Option<u32>,
    pub weight: Weight,
}

impl Hash for LabelEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex.hash(state);
    }
}

impl PartialEq for LabelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.vertex.eq(&other.vertex)
    }
}

impl Eq for LabelEntry {}
