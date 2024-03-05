use core::panic;
use std::usize;

use ahash::{HashSet, HashSetExt};
use serde_derive::{Deserialize, Serialize};

use crate::graphs::{path::Path, types::VertexId};

use super::label_entry::LabelEntry;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Label {
    pub entries: Vec<LabelEntry>,
}

impl Label {
    pub fn new(vertex: VertexId) -> Label {
        Label {
            entries: vec![LabelEntry::new(vertex)],
        }
    }

    pub fn get_path(&self, edge_id: u32) -> Path {
        let mut path = Path {
            vertices: Vec::new(),
            weight: self.entries[edge_id as usize].weight,
        };
        let mut current_idx = edge_id;
        let mut visited = HashSet::new();

        while let Some(entry) = self.entries.get(current_idx as usize) {
            // cycle detection
            if !visited.insert(current_idx) {
                panic!("wrong formated label");
            }

            path.vertices.push(entry.vertex);

            if let Some(this_idx) = entry.predecessor {
                current_idx = this_idx;
            } else {
                // exit the loop if there's no predecessor
                break;
            }
        }

        path
    }
}
