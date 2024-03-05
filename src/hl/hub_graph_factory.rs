use ahash::{HashMap, HashMapExt};
use indicatif::ProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    ch::fast_shortcut_replacer::FastShortcutReplacer, graphs::types::VertexId,
    simple_algorithms::ch_bi_dijkstra::ChDijkstra,
};

use super::{hub_graph::HubGraph, label::Label};

pub struct HubGraphFactory<'a> {
    pub ch_dijkstra: &'a ChDijkstra,
}

impl<'a> HubGraphFactory<'a> {
    pub fn new(ch_dijkstra: &'a ChDijkstra) -> HubGraphFactory {
        HubGraphFactory { ch_dijkstra }
    }

    pub fn get_hl(&self) -> HubGraph {
        let mut forward_labels: Vec<_> = (0..self.ch_dijkstra.graph.num_nodes())
            .map(|vertex| Label::new(vertex))
            .collect();

        let mut reverse_labels = forward_labels.clone();

        for level_list in self.ch_dijkstra.levels.iter().rev().progress() {
            for vertex in level_list {
                let forward_label = self.create_label(*vertex, &forward_labels, &reverse_labels);
                let reverse_label = self.create_label(*vertex, &reverse_labels, &forward_labels);

                forward_labels[*vertex as usize] = forward_label;
                reverse_labels[*vertex as usize] = reverse_label;
            }
        }
        let shortcut_replacer = FastShortcutReplacer::new(&self.ch_dijkstra.shortcuts);

        // Needs to be called after all labels are creates as replacing the predecessor VertexId
        // with the index of predecessor in label makes merging impossible.
        forward_labels
            .iter_mut()
            .chain(reverse_labels.iter_mut())
            .for_each(|label| Self::set_predecessor(label));

        HubGraph {
            forward_labels,
            reverse_labels,
            shortcut_replacer,
        }
    }

    /// Creates the forward or reverse label for `vertex`.
    ///
    /// If direction1 == forward and direction2 == reverse, the forward label is created. If the
    /// directions are switched, the reverse label is created.
    fn create_label(
        &self,
        vertex: VertexId,
        direction1_labels: &Vec<Label>,
        direction2_labels: &Vec<Label>,
    ) -> Label {
        let mut labels = Vec::new();
        for out_edge in self.ch_dijkstra.graph.out_edges(vertex) {
            let mut label = direction1_labels[out_edge.head as usize].clone();
            label.entries.iter_mut().for_each(|entry| {
                entry.predecessor.get_or_insert(vertex);
                entry.weight += out_edge.weight
            });
            labels.push(label);
        }
        let mut label = Self::merge(labels, vertex);
        Self::prune(&mut label, direction2_labels);
        label
    }

    pub fn set_predecessor(label: &mut Label) {
        // maps vertex -> index
        let mut vertex_to_index = HashMap::new();
        for idx in 0..label.entries.len() {
            vertex_to_index.insert(label.entries[idx].vertex, idx as u32);
        }

        // replace predecessor VertexId with index of predecessor
        for entry in label.entries.iter_mut() {
            if let Some(predecessor) = entry.predecessor {
                entry.predecessor = Some(*vertex_to_index.get(&predecessor).unwrap());
            }
        }
    }

    pub fn merge(mut labels: Vec<Label>, vertex: VertexId) -> Label {
        labels.iter_mut().for_each(|label| label.entries.reverse());
        let mut label_entries = Vec::new();

        labels.push(Label::new(vertex));

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

    pub fn prune(label: &mut Label, reverse_labels: &[Label]) {
        label.entries = label
            .entries
            .par_iter()
            .filter(|entry| {
                let reverse_label = &reverse_labels[entry.vertex as usize];
                let true_cost = HubGraph::get_weight_labels(label, reverse_label).unwrap();
                entry.weight == true_cost
            })
            .cloned()
            .collect();
    }
}
