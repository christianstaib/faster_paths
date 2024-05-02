use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use ahash::{HashMap, HashMapExt};
use dashmap::DashMap;
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
    graphs::{
        edge::DirectedEdge,
        graph_functions::{shortests_path_tree, validate_path},
        path::ShortestPathTestCase,
        Graph, VertexId,
    },
    hl::hl_path_finding::shortest_path,
    shortcut_replacer::slow_shortcut_replacer::replace_shortcuts_slow,
};

pub fn generate_hub_graph(
    graph: &dyn Graph,
    order: &[u32],
) -> (HubGraph, HashMap<DirectedEdge, VertexId>) {
    let shortcuts: Arc<DashMap<DirectedEdge, VertexId>> = Arc::new(DashMap::new());

    println!("generating labels");
    let labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, label_shortcuts) = generate_forward_label(vertex, graph, order);

            for (edge, vertex_id) in label_shortcuts {
                // DashMap's entry API can be used to efficiently check and update the map
                shortcuts.entry(edge).or_insert(vertex_id);
            }

            label
        })
        .collect();

    println!("generating shortcut map");
    let mut shortcuts: HashMap<DirectedEdge, VertexId> =
        Arc::into_inner(shortcuts).unwrap().into_iter().collect();

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
            let (label, mut label_shortcuts) = generate_forward_label(vertex, graph, order);

            // Spend as little time as possible in a locked shortcuts state, therefore
            // remove label_shortcuts already present in shortcuts in readmode.
            // Important to put readable_shortcuts into its own scope as it would otherwise
            // block write access
            let start = Instant::now();
            if let Ok(readable_shortcuts) = shortcuts.read() {
                let elapsed = start.elapsed().as_secs_f32();
                if elapsed > 1.0 {
                    println!("waited {}s for read lock", elapsed);
                }
                label_shortcuts.retain(|(edge, _)| !readable_shortcuts.contains_key(&edge));
            }

            if !label_shortcuts.is_empty() {
                let start = Instant::now();
                if let Ok(ref mut shortcuts) = shortcuts.write() {
                    let elapsed = start.elapsed().as_secs_f32();
                    if elapsed > 1.0 {
                        println!("waited {}s for write lock", elapsed);
                    }
                    shortcuts.extend(label_shortcuts);
                }
            }

            label
        })
        .collect();

    println!("generating reverse labels");
    let reverse_labels: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map(|vertex| {
            let (label, mut label_shortcuts) = generate_reverse_label(vertex, graph, order);

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

pub fn generate_forward_label(
    vertex: VertexId,
    graph: &dyn Graph,
    order: &[u32],
) -> (Label, Vec<(DirectedEdge, VertexId)>) {
    let data = Dijkstra::new(graph).single_source(vertex);
    get_label_from_data(vertex, &data, order)
}

pub fn generate_reverse_label(
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

pub fn predict_average_label_size(
    test_cases: &Vec<ShortestPathTestCase>,
    number_of_labels: u32,
    graph: &dyn Graph,
    order: &Vec<u32>,
) -> f64 {
    let labels: Vec<_> = test_cases
        .par_iter()
        .take(number_of_labels as usize)
        .progress()
        .map(|test_case| {
            let mut shortcuts = HashMap::new();

            let (forward_label, forward_shortcuts) =
                generate_forward_label(test_case.request.source(), graph, order);
            let (reverse_label, reverse_shortcuts) =
                generate_reverse_label(test_case.request.target(), graph, order);

            shortcuts.extend(forward_shortcuts.iter().cloned());
            shortcuts.extend(
                reverse_shortcuts
                    .into_iter()
                    .map(|(x, y)| (x.reversed(), y)),
            );

            let mut path = shortest_path(&forward_label, &reverse_label);

            // if there exits a path, replace the shortcuts on it
            if let Some(ref mut path) = path {
                replace_shortcuts_slow(&mut path.vertices, &shortcuts);
            }

            if let Err(err) = validate_path(graph, test_case, &path) {
                panic!("{}", err);
            }

            vec![forward_label, reverse_label]
        })
        .flatten()
        .collect();

    let average_label_size =
        labels.iter().map(|l| l.entries.len()).sum::<usize>() as f64 / labels.len() as f64;
    average_label_size
}
