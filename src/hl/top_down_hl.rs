use std::sync::{Arc, RwLock};

use ahash::{HashMap, HashMapExt};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use super::{
    hl_from_ch::set_predecessor,
    hub_graph::{DirectedHubGraph, HubGraph},
    label::{Label, LabelEntry},
};
use crate::{
    classical_search::dijkstra::Dijkstra,
    dijkstra_data::dijkstra_data_vec::DijkstraDataVec,
    graphs::{edge::DirectedEdge, graph_functions::shortests_path_tree, Graph, VertexId},
};

pub fn generate_hub_graph(
    graph: &dyn Graph,
    order: &[u32],
) -> (HubGraph, HashMap<DirectedEdge, VertexId>) {
    let shortcuts: Arc<RwLock<HashMap<DirectedEdge, VertexId>>> =
        Arc::new(RwLock::new(HashMap::new()));

    println!("generating labels");
    let labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, mut label_shortcuts) = get_out_label(vertex, graph, order);

            // Spend as little time as possible in a locked shortcuts state, therefore
            // remove label_shortcuts already present in shortcuts in readmode.
            // Important to put readable_shortcuts into its own scope as it would otherwise
            // block write access
            if let Ok(readable_shortcuts) = shortcuts.read() {
                label_shortcuts.retain(|(edge, _)| !readable_shortcuts.contains_key(&edge));
            }

            if !label_shortcuts.is_empty() {
                shortcuts.write().unwrap().extend(label_shortcuts);
            }

            label
        })
        .collect();

    println!("generating shortcut map");
    let mut shortcuts: HashMap<DirectedEdge, VertexId> =
        shortcuts.read().unwrap().to_owned().into_iter().collect();

    // the shortcuts for the reverse direction have not yet been
    // added.
    let reverse_shortcuts: Vec<_> = shortcuts
        .par_iter()
        .map(|(edge, &vertex)| (edge.reversed(), vertex))
        .filter(|(edge, _)| !shortcuts.contains_key(edge))
        .collect();
    shortcuts.extend(reverse_shortcuts);

    let hub_graph = HubGraph { labels };

    (hub_graph, shortcuts)
}

pub fn generate_directed_hub_graph(
    graph: &dyn Graph,
    order: &[u32],
) -> (DirectedHubGraph, HashMap<DirectedEdge, VertexId>) {
    let shortcuts: Arc<RwLock<HashMap<DirectedEdge, VertexId>>> =
        Arc::new(RwLock::new(HashMap::new()));

    println!("generating forward labels");
    let forward_labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, mut label_shortcuts) = get_out_label(vertex, graph, order);

            // Spend as little time as possible in a locked shortcuts state, therefore
            // remove label_shortcuts already present in shortcuts in readmode.
            // Important to put readable_shortcuts into its own scope as it would otherwise
            // block write access
            if let Ok(readable_shortcuts) = shortcuts.read() {
                label_shortcuts.retain(|(edge, _)| !readable_shortcuts.contains_key(&edge));
            }

            if !label_shortcuts.is_empty() {
                shortcuts.write().unwrap().extend(label_shortcuts);
            }

            label
        })
        .collect();

    println!("generating reverse labels");
    let reverse_labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, mut label_shortcuts) = get_in_label(vertex, graph, order);

            if let Ok(readable_shortcuts) = shortcuts.read() {
                label_shortcuts.retain(|(edge, _)| !readable_shortcuts.contains_key(&edge));
            }

            if !label_shortcuts.is_empty() {
                shortcuts.write().unwrap().extend(label_shortcuts);
            }

            label
        })
        .collect();

    println!("getting shortcuts vec");
    let shortcuts: HashMap<DirectedEdge, VertexId> =
        shortcuts.read().unwrap().to_owned().into_iter().collect();

    let directed_hub_graph = DirectedHubGraph {
        forward_labels,
        reverse_labels,
    };

    (directed_hub_graph, shortcuts)
}

pub fn get_out_label(
    vertex: VertexId,
    graph: &dyn Graph,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let data = Dijkstra::new(graph).single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

pub fn get_in_label(
    vertex: VertexId,
    graph: &dyn Graph,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let data = Dijkstra::new(graph).single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

pub fn get_label_from_data(
    vertex: VertexId,
    data: &DijkstraDataVec,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let mut shortest_path_tree = shortests_path_tree(data);
    let mut shortcuts = Vec::new();

    let mut stack = vec![vertex as usize];

    let mut label = Label::new(vertex);
    while let Some(tail) = stack.pop() {
        let mut heads = std::mem::take(&mut shortest_path_tree[tail]);

        while let Some(head) = heads.pop() {
            if order[head as usize] > order[tail] {
                stack.push(head as usize);
                label.entries.push(LabelEntry {
                    vertex: head,
                    predecessor: Some(tail as VertexId),
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

    label.entries.sort_unstable_by_key(|entry| entry.vertex);
    label.entries.shrink_to_fit();
    set_predecessor(&mut label);

    (label, shortcuts)
}
