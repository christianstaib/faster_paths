use std::collections::BinaryHeap;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    ch::{contractor::ContractedGraph, fast_shortcut_replacer::FastShortcutReplacer},
    graphs::{
        edge::DirectedEdge,
        fast_graph::FastGraph,
        path::{Path, PathRequest},
        types::VertexId,
    },
    hl::{hub_graph::HubGraph, label::Label, label_entry::LabelEntry},
    queue::heap_queue::State,
};

#[derive(Clone)]
pub struct ChDijkstra {
    pub graph: FastGraph,
    pub shortcuts: HashMap<DirectedEdge, u32>,
    pub levels: Vec<Vec<VertexId>>,
}

impl ChDijkstra {
    pub fn new(contracted_grap: &ContractedGraph) -> ChDijkstra {
        let shortcuts = contracted_grap.shortcuts_map.iter().cloned().collect();
        let graph = FastGraph::from_graph(&contracted_grap.graph);
        ChDijkstra {
            graph,
            shortcuts,
            levels: contracted_grap.levels.clone(),
        }
    }

    pub fn get_hl(&self) -> HubGraph {
        let mut out_labels: Vec<_> = (0..self.graph.num_nodes())
            .map(|vertex| {
                let entry = LabelEntry {
                    vertex,
                    weight: 0,
                    predecessor: None,
                };

                Label {
                    entries: vec![entry],
                }
            })
            .collect();

        let mut in_labels = out_labels.clone();

        for level_list in self.levels.iter().rev().progress() {
            for vertex in level_list {
                let mut labels = Vec::new();
                for out_edge in self.graph.out_edges(*vertex) {
                    let mut head_label_entries = out_labels[out_edge.head as usize].entries.clone();
                    head_label_entries.iter_mut().for_each(|entry| {
                        if entry.vertex == out_edge.head {
                            entry.predecessor = Some(*vertex);
                        }
                        entry.weight += out_edge.weight
                    });
                    let label = Label {
                        entries: head_label_entries.clone(),
                    };
                    labels.push(label);

                    out_labels[*vertex as usize]
                        .entries
                        .extend(head_label_entries);
                }
                let new_label = Label::merge(labels, *vertex);
                out_labels[*vertex as usize].clean();
                assert_eq!(new_label, out_labels[*vertex as usize]);
                out_labels[*vertex as usize].prune_forward_label(&in_labels);

                for in_edge in self.graph.in_edges(*vertex) {
                    let mut tail_label_entries = in_labels[in_edge.tail as usize].entries.clone();
                    tail_label_entries.iter_mut().for_each(|entry| {
                        if entry.vertex == in_edge.tail {
                            entry.predecessor = Some(*vertex);
                        }
                        entry.weight += in_edge.weight
                    });

                    in_labels[*vertex as usize]
                        .entries
                        .extend(tail_label_entries);
                }
                in_labels[*vertex as usize].clean();
                in_labels[*vertex as usize].prune_reverse_label(&out_labels);
            }
        }
        let shortcut_replacer = FastShortcutReplacer::new(&self.shortcuts);

        out_labels
            .par_iter_mut()
            .chain(in_labels.par_iter_mut())
            .progress()
            .for_each(|label| label.set_predecessor());

        HubGraph {
            forward_labels: out_labels,
            reverse_labels: in_labels,
            shortcut_replacer,
        }
    }

    /// (contact_node, cost)
    pub fn get_cost(&self, request: &PathRequest) -> Option<u32> {
        let mut forward_costs = HashMap::new();
        let mut backward_costs = HashMap::new();

        let mut forward_open = BinaryHeap::new();
        let mut backward_open = BinaryHeap::new();

        let mut forward_expanded = HashSet::new();
        let mut backward_expaned = HashSet::new();

        forward_open.push(State {
            key: 0,
            value: request.source,
        });
        forward_costs.insert(request.source, 0);

        backward_open.push(State {
            key: 0,
            value: request.target,
        });
        backward_costs.insert(request.target, 0);

        let mut minimal_cost = u32::MAX;

        while !forward_open.is_empty() || !backward_open.is_empty() {
            if let Some(forward_state) = forward_open.pop() {
                let current_node = forward_state.value;
                if !forward_expanded.contains(&current_node) {
                    forward_expanded.insert(current_node);

                    if backward_expaned.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                        }
                    }

                    self.graph
                        .out_edges(forward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                forward_costs.get(&current_node).unwrap() + edge.weight;
                            let current_cost = *forward_costs.get(&edge.head).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                forward_costs.insert(edge.head, alternative_cost);
                                forward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.head,
                                });
                            }
                        });
                }
            }

            if let Some(backward_state) = backward_open.pop() {
                let current_node = backward_state.value;
                if !backward_expaned.contains(&current_node) {
                    backward_expaned.insert(current_node);

                    if forward_expanded.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                        }
                    }

                    self.graph
                        .in_edges(backward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                backward_costs.get(&current_node).unwrap() + edge.weight;
                            let current_cost = *backward_costs.get(&edge.tail).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                backward_costs.insert(edge.tail, alternative_cost);
                                backward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.tail,
                                });
                            }
                        });
                }
            }
        }

        if minimal_cost != u32::MAX {
            return Some(minimal_cost);
        }
        None
    }

    /// (contact_node, cost)
    pub fn get_route(&self, request: &PathRequest) -> Option<Path> {
        let mut forward_costs = HashMap::new();
        let mut backward_costs = HashMap::new();

        let mut forward_predecessor = HashMap::new();
        let mut backward_predecessor = HashMap::new();

        let mut forward_open = BinaryHeap::new();
        let mut backward_open = BinaryHeap::new();

        let mut forward_expanded = HashSet::new();
        let mut backward_expaned = HashSet::new();

        forward_open.push(State {
            key: 0,
            value: request.source,
        });
        forward_costs.insert(request.source, 0);

        backward_open.push(State {
            key: 0,
            value: request.target,
        });
        backward_costs.insert(request.target, 0);

        let mut minimal_cost = u32::MAX;
        let mut meeting_node = u32::MAX;

        while !forward_open.is_empty() || !backward_open.is_empty() {
            if let Some(forward_state) = forward_open.pop() {
                let current_node = forward_state.value;
                if !forward_expanded.contains(&current_node) {
                    forward_expanded.insert(current_node);

                    if backward_expaned.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                            meeting_node = forward_state.value;
                        }
                    }

                    self.graph
                        .out_edges(forward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                forward_costs.get(&current_node).unwrap() + edge.weight;
                            let current_cost = *forward_costs.get(&edge.head).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                forward_costs.insert(edge.head, alternative_cost);
                                forward_predecessor.insert(edge.head, current_node);
                                forward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.head,
                                });
                            }
                        });
                }
            }

            if let Some(backward_state) = backward_open.pop() {
                let current_node = backward_state.value;
                if !backward_expaned.contains(&current_node) {
                    backward_expaned.insert(current_node);

                    if forward_expanded.contains(&current_node) {
                        let contact_cost = forward_costs.get(&current_node).unwrap()
                            + backward_costs.get(&current_node).unwrap();
                        if contact_cost < minimal_cost {
                            minimal_cost = contact_cost;
                            meeting_node = backward_state.value;
                        }
                    }

                    self.graph
                        .in_edges(backward_state.value)
                        .iter()
                        .for_each(|edge| {
                            let alternative_cost =
                                backward_costs.get(&current_node).unwrap() + edge.weight;
                            let current_cost = *backward_costs.get(&edge.tail).unwrap_or(&u32::MAX);
                            if alternative_cost < current_cost {
                                backward_costs.insert(edge.tail, alternative_cost);
                                backward_predecessor.insert(edge.tail, current_node);
                                backward_open.push(State {
                                    key: alternative_cost,
                                    value: edge.tail,
                                });
                            }
                        });
                }
            }
        }

        get_route(
            meeting_node,
            minimal_cost,
            forward_predecessor,
            backward_predecessor,
        )
    }
}

fn get_route(
    meeting_node: u32,
    meeting_cost: u32,
    forward_predecessor: HashMap<u32, u32>,
    backward_predecessor: HashMap<u32, u32>,
) -> Option<Path> {
    if meeting_cost == u32::MAX {
        return None;
    }
    let mut route = Vec::new();
    let mut current = meeting_node;
    route.push(current);
    while let Some(new_current) = forward_predecessor.get(&current) {
        current = *new_current;
    }
    current = meeting_node;
    while let Some(new_current) = backward_predecessor.get(&current) {
        current = *new_current;
    }
    let route = Path {
        vertices: route,
        weight: meeting_cost,
    };
    Some(route)
}
