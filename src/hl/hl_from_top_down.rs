use std::sync::{Arc, RwLock};

use ahash::{HashMap, HashMapExt};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rayon::prelude::*;

use super::{
    directed_hub_graph::DirectedHubGraph,
    hl_from_ch::set_predecessor,
    label::{new_label, LabelEntry},
};
use crate::{
    classical_search::dijkstra::{single_source, single_target},
    dijkstra_data::dijkstra_data_vec::DijkstraDataVec,
    graphs::{
        edge::DirectedEdge,
        graph_functions::{shortests_path_tree},
        Graph, VertexId,
    },
};

pub fn generate_directed_hub_graph(graph: &dyn Graph, order: &[u32]) -> DirectedHubGraph {
    let shortcuts = Arc::new(RwLock::new(HashMap::new()));

    println!("generating forward labels");
    let forward_labels = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, mut label_shortcuts) = generate_forward_label(vertex, graph, order);

            // Spend as little time as possible in a locked shortcuts state, therefore
            // remove label_shortcuts already present in shortcuts in readmode.
            // block write access
            if let Ok(shortcuts) = shortcuts.read() {
                label_shortcuts.retain(|(edge, _)| !shortcuts.contains_key(edge));
            }

            if !label_shortcuts.is_empty() {
                if let Ok(ref mut shortcuts) = shortcuts.write() {
                    shortcuts.extend(label_shortcuts);
                }
            }

            label
        })
        .collect();

    println!("generating reverse labels");
    let backward_labels = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, mut label_shortcuts) = generate_backward_label(vertex, graph, order);

            if let Ok(shortcuts) = shortcuts.read() {
                label_shortcuts.retain(|(edge, _)| !shortcuts.contains_key(edge));
            }

            if !label_shortcuts.is_empty() {
                shortcuts.write().unwrap().extend(label_shortcuts);
            }

            label
        })
        .collect();

    println!("getting shortcuts vec");
    let shortcuts = shortcuts
        .read()
        .unwrap()
        .to_owned()
        .into_iter()
        .progress()
        .collect();

    DirectedHubGraph::new(forward_labels, backward_labels, shortcuts)
}

pub fn generate_forward_label(
    vertex: VertexId,
    graph: &dyn Graph,
    vertex_to_level_map: &[u32],
) -> (Vec<LabelEntry>, Vec<(DirectedEdge, VertexId)>) {
    let data = single_source(graph, vertex);
    get_label_from_data(vertex, &data, vertex_to_level_map)
}

pub fn generate_backward_label(
    vertex: VertexId,
    graph: &dyn Graph,
    vertex_to_level_map: &[u32],
) -> (Vec<LabelEntry>, Vec<(DirectedEdge, VertexId)>) {
    let data = single_target(graph, vertex);
    let (label, shortcuts) = get_label_from_data(vertex, &data, vertex_to_level_map);
    (
        label,
        shortcuts
            .into_iter()
            .map(|(edge, vertex)| (edge.reversed(), vertex))
            .collect_vec(),
    )
}

pub fn get_label_from_data(
    vertex: VertexId,
    data: &DijkstraDataVec,
    vertex_to_level_map: &[u32],
) -> (Vec<LabelEntry>, Vec<(DirectedEdge, VertexId)>) {
    let mut shortest_path_tree = shortests_path_tree(data);
    let mut shortcuts = Vec::new();

    let mut stack = vec![vertex as usize];

    let mut label = new_label(vertex);
    while let Some(tail) = stack.pop() {
        let mut heads = std::mem::take(&mut shortest_path_tree[tail]);

        while let Some(head) = heads.pop() {
            if vertex_to_level_map[head as usize] > vertex_to_level_map[tail] {
                stack.push(head as usize);
                label.push(LabelEntry {
                    vertex: head,
                    predecessor_index: tail as VertexId,
                    weight: data.vertices[head as usize].weight.unwrap(),
                });
            } else {
                for &tail_child in std::mem::take(&mut shortest_path_tree[head as usize]).iter() {
                    heads.push(tail_child);
                    let edge = DirectedEdge::new(tail as VertexId, tail_child).unwrap();
                    shortcuts.push((edge, head));
                }
            }
        }
    }

    label.sort_unstable_by_key(|entry| entry.vertex);
    label.shrink_to_fit();
    set_predecessor(&mut label);

    (label, shortcuts)
}
