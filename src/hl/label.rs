use std::usize;

use serde_derive::{Deserialize, Serialize};

use crate::graphs::{path::Path, VertexId};

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

    pub fn get_path(&self, entry_index: u32) -> Option<Path> {
        let mut path = Path {
            vertices: Vec::new(),
            weight: self.entries.get(entry_index as usize)?.weight,
        };
        let mut current_index = entry_index;

        while let Some(entry) = self.entries.get(current_index as usize) {
            path.vertices.push(entry.vertex);

            // cycle detection
            if path.vertices.len() > self.entries.len() {
                return None;
            }

            // exit the loop if there's no predecessor
            if let Some(predecessor_index) = entry.predecessor {
                current_index = predecessor_index;
            } else {
                break;
            }
        }

        Some(path)
    }
}
