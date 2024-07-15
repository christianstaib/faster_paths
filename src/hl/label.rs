use std::usize;

use serde::{Deserialize, Serialize};

use crate::graphs::{path::Path, VertexId, Weight};

pub fn new_label(vertex: VertexId) -> Vec<LabelEntry> {
    vec![LabelEntry::new(vertex)]
}

pub fn get_path(label: &[LabelEntry], entry_index: u32) -> Option<Path> {
    let mut path = Path {
        vertices: Vec::new(),
        weight: label.get(entry_index as usize)?.weight,
    };
    let mut current_index = entry_index;

    while let Some(entry) = label.get(current_index as usize) {
        path.vertices.push(entry.vertex);

        // cycle detection
        if path.vertices.len() > label.len() {
            panic!("label is incorrect");
        }

        // exit the loop if there's no predecessor
        if entry.predecessor_index != u32::MAX {
            current_index = entry.predecessor_index;
        } else {
            break;
        }
    }

    Some(path)
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub vertex: VertexId,
    pub predecessor_index: u32,
    pub weight: Weight,
}

impl LabelEntry {
    pub fn new(vertex: VertexId) -> LabelEntry {
        LabelEntry {
            vertex,
            predecessor_index: u32::MAX,
            weight: 0,
        }
    }
}
