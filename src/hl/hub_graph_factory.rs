use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    ch::fast_shortcut_replacer::FastShortcutReplacer, simple_algorithms::ch_bi_dijkstra::ChDijkstra,
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
                let forward_label = self.forward_label(vertex, &forward_labels, &reverse_labels);
                let reverse_label = self.reverse_label(vertex, &reverse_labels, &forward_labels);

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
            .for_each(|label| label.set_predecessor());

        HubGraph {
            forward_labels,
            reverse_labels,
            shortcut_replacer,
        }
    }

    fn reverse_label(
        &self,
        vertex: &u32,
        forward_labels: &Vec<Label>,
        reverse_labels: &Vec<Label>,
    ) -> Label {
        let mut labels = Vec::new();
        for in_edge in self.ch_dijkstra.graph.in_edges(*vertex) {
            let mut label = forward_labels[in_edge.tail as usize].clone();
            label.entries.iter_mut().for_each(|entry| {
                entry.predecessor.get_or_insert(*vertex);
                entry.weight += in_edge.weight
            });
            labels.push(label);
        }
        let mut label = Label::merge(labels, *vertex);
        label.prune_reverse_label(reverse_labels);
        label
    }

    fn forward_label(
        &self,
        vertex: &u32,
        forward_labels: &Vec<Label>,
        reverse_labels: &Vec<Label>,
    ) -> Label {
        let mut labels = Vec::new();
        for out_edge in self.ch_dijkstra.graph.out_edges(*vertex) {
            let mut label = forward_labels[out_edge.head as usize].clone();
            label.entries.iter_mut().for_each(|entry| {
                entry.predecessor.get_or_insert(*vertex);
                entry.weight += out_edge.weight
            });
            labels.push(label);
        }
        let mut label = Label::merge(labels, *vertex);
        label.prune_forward_label(reverse_labels);
        label
    }
}
