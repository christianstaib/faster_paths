use ahash::{HashMap, HashMapExt};
use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    ch::contracted_graph::ContractedGraph,
    graphs::{Graph, VertexId},
};

use super::{
    hub_graph::{overlap, HubGraph},
    label::Label,
};

pub struct HubGraphFactory<'a> {
    pub ch_information: &'a ContractedGraph,
}

impl<'a> HubGraphFactory<'a> {
    pub fn new(ch_information: &'a ContractedGraph) -> HubGraphFactory {
        HubGraphFactory { ch_information }
    }

    pub fn get_hl(&self) -> HubGraph {
        let mut forward_labels: Vec<_> = (0..self.ch_information.upward_graph.number_of_vertices())
            .map(Label::new)
            .collect();

        let mut reverse_labels = forward_labels.clone();

        let pb = ProgressBar::new(self.ch_information.upward_graph.number_of_vertices() as u64);
        for level_list in self.ch_information.levels.iter().rev() {
            let labels: Vec<_> = level_list
                .par_iter()
                .map(|&vertex| {
                    let forward_label =
                        self.generate_forward_label(vertex, &forward_labels, &reverse_labels);
                    let reverse_label =
                        self.generate_reverse_label(vertex, &forward_labels, &reverse_labels);

                    (vertex, forward_label, reverse_label)
                })
                .collect();
            for (vertex, forward_label, reverse_label) in labels {
                forward_labels[vertex as usize] = forward_label;
                reverse_labels[vertex as usize] = reverse_label;
            }
            pb.inc(level_list.len() as u64);
        }
        pb.finish();

        // Needs to be called after all labels are creates as replacing the predecessor VertexId
        // with the index of predecessor in label makes merging impossible.
        forward_labels
            .iter_mut()
            .chain(reverse_labels.iter_mut())
            .for_each(Self::set_predecessor);

        HubGraph {
            forward_labels,
            reverse_labels,
        }
    }

    /// Generates a forward label for a given vertex.
    ///
    /// This function constructs a forward label for the specified vertex by considering outgoing edges. It accumulates
    /// the weights and predecessors from the labels associated with the direction towards the destination (forward) and
    /// adjusts them based on the graph's structure. The resulting label is optimized by merging similar paths and
    /// pruning paths that are not efficient when considered with the reverse direction paths.
    fn generate_forward_label(
        &self,
        vertex: VertexId,
        forward_labels: &Vec<Label>,
        reverse_labels: &Vec<Label>,
    ) -> Label {
        let mut labels = Vec::new();
        for out_edge in self.ch_information.upward_graph.out_edges(vertex) {
            let mut label = forward_labels[out_edge.head() as usize].clone();
            label.entries.iter_mut().for_each(|entry| {
                entry.predecessor.get_or_insert(vertex);
                entry.weight += out_edge.weight();
            });
            labels.push(label);
        }
        let mut label = Self::merge(labels, vertex);
        Self::prune(&mut label, reverse_labels);
        label
    }

    /// Generates a reverse label for a given vertex.
    ///
    /// This function constructs a reverse label for the specified vertex by considering incoming edges. It accumulates
    /// the weights and predecessors from the labels associated with the direction from the source (reverse) and
    /// adjusts them based on the graph's structure. The resulting label is optimized by merging similar paths and
    /// pruning paths that are not efficient when considered with the forward direction paths.
    fn generate_reverse_label(
        &self,
        vertex: VertexId,
        forward_labels: &Vec<Label>,
        reverse_labels: &Vec<Label>,
    ) -> Label {
        let mut labels = Vec::new();
        for in_edge in self.ch_information.downward_graph.out_edges(vertex) {
            let mut label = reverse_labels[in_edge.head() as usize].clone();
            label.entries.iter_mut().for_each(|entry| {
                entry.predecessor.get_or_insert(vertex);
                entry.weight += in_edge.weight();
            });
            labels.push(label);
        }
        let mut label = Self::merge(labels, vertex);
        Self::prune(&mut label, forward_labels);
        label
    }

    fn set_predecessor(label: &mut Label) {
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

    fn merge(mut labels: Vec<Label>, vertex: VertexId) -> Label {
        // poping from end of vec is faster as poping from beginning
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

    fn prune(direction1_label: &mut Label, direction2_labels_labels: &[Label]) {
        direction1_label.entries = direction1_label
            .entries
            .par_iter()
            .filter(|entry| {
                let reverse_label = &direction2_labels_labels[entry.vertex as usize];
                let true_weight = overlap(direction1_label, reverse_label).unwrap().0;
                entry.weight == true_weight
            })
            .cloned()
            .collect();
    }
}
