use serde_derive::{Deserialize, Serialize};

use crate::graphs::types::{VertexId, Weight};

#[derive(Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub vertex: VertexId,
    pub predecessor: Option<u32>,
    pub weight: Weight,
}
