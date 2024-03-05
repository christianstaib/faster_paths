use core::panic;
use std::usize;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::graphs::{path::Path, types::VertexId};

use super::{hub_graph::HubGraph, label_entry::LabelEntry};

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

    pub fn prune(&mut self, reverse_labels: &[Label]) {
        self.entries = self
            .entries
            .par_iter()
            .filter(|entry| {
                let reverse_label = &reverse_labels[entry.vertex as usize];
                let true_cost = HubGraph::get_weight_labels(self, reverse_label).unwrap();
                entry.weight == true_cost
            })
            .cloned()
            .collect();
    }

    pub fn set_predecessor(&mut self) {
        // maps vertex -> index
        let mut vertex_to_index = HashMap::new();
        for idx in 0..self.entries.len() {
            vertex_to_index.insert(self.entries[idx].vertex, idx as u32);
        }

        // replace predecessor VertexId with index of predecessor
        for entry in self.entries.iter_mut() {
            if let Some(predecessor) = entry.predecessor {
                entry.predecessor = Some(*vertex_to_index.get(&predecessor).unwrap());
            }
        }
    }

    pub fn merge(mut labels: Vec<Label>, vertex: VertexId) -> Label {
        labels.iter_mut().for_each(|label| label.entries.reverse());
        let mut label_entries = Vec::new();

        labels.push(Label {
            entries: vec![LabelEntry {
                vertex,
                predecessor: None,
                weight: 0,
            }],
        });

        while !labels.is_empty() {
            let min_vertex = labels
                .iter()
                .map(|label| label.entries.last().unwrap().vertex)
                .min()
                .unwrap();
            let entries: Vec<_> = labels
                .iter_mut()
                .filter_map(|label| {
                    if label.entries.last().unwrap().vertex == min_vertex {
                        return label.entries.pop();
                    }
                    None
                })
                .collect();
            labels.retain(|label| !label.entries.is_empty());
            let min_entry = entries
                .into_iter()
                .min_by_key(|entry| entry.weight)
                .unwrap();
            label_entries.push(min_entry);
        }

        Label {
            entries: label_entries,
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
