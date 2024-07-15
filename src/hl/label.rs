use std::usize;

use serde::{Deserialize, Serialize};

use crate::graphs::{path::Path, VertexId, Weight};

pub fn new_label(vertex: VertexId) -> Vec<LabelEntry> {
    vec![LabelEntry::new(vertex)]
}

pub fn get_path(label: &[LabelEntry], entry_index: u32) -> Option<Path> {
    let weight = label.get(entry_index as usize)?.weight;
    let mut vertices = Vec::new();

    let mut index_of_current_vertex = entry_index;
    loop {
        let entry = &label[index_of_current_vertex as usize];

        vertices.push(entry.vertex);

        // cycle detection
        if vertices.len() > label.len() {
            panic!("label is incorrect");
        }

        // exit the loop if there's no predecessor
        if entry.predecessor_index != u32::MAX {
            index_of_current_vertex = entry.predecessor_index;
        } else {
            break;
        }
    }

    Some(Path { vertices, weight })
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LabelEntry {
    pub vertex: VertexId,
    pub weight: Weight,
    pub predecessor_index: u32,
}

impl LabelEntry {
    pub fn new(vertex: VertexId) -> LabelEntry {
        LabelEntry {
            vertex,
            weight: 0,
            predecessor_index: u32::MAX,
        }
    }
}
