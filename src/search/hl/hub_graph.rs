use std::{cmp::Ordering, collections::HashMap};

use indicatif::ProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use super::half_hub_graph::{get_hub_label_by_merging, set_predecessor, HalfHubGraph};
use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Level, Vertex},
    search::{
        ch::contracted_graph::{vertex_to_level, ContractedGraph},
        collections::dijkstra_data::Path,
        shortcuts::replace_shortcuts_slowly,
        PathFinding,
    },
    utility::get_progressbar_long_jobs,
};

#[derive(Serialize, Deserialize)]
pub struct HubGraph {
    pub forward: HalfHubGraph,
    pub backward: HalfHubGraph,
    shortcuts: HashMap<(Vertex, Vertex), Vertex>,
    level_to_vertex: Vec<Vertex>,
    vertex_to_level: Vec<Level>,
}

impl HubGraph {
    pub fn by_brute_force<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        level_to_vertex: &Vec<Vertex>,
    ) -> HubGraph {
        let vertex_to_level = vertex_to_level(level_to_vertex);

        let (forward, mut shortcuts) = HalfHubGraph::by_brute_force(
            graph.out_graph(),
            &vertex_to_level,
            get_progressbar_long_jobs(
                "Brute forcing forward labels",
                graph.out_graph().number_of_vertices() as u64,
            ),
        );
        let (backward, backward_shortcuts) = HalfHubGraph::by_brute_force(
            graph.in_graph(),
            &vertex_to_level,
            get_progressbar_long_jobs(
                "Brute forcing backward labels",
                graph.out_graph().number_of_vertices() as u64,
            ),
        );

        for ((tail, head), skiped_vertex) in backward_shortcuts.into_iter() {
            shortcuts.insert((head, tail), skiped_vertex);
        }

        HubGraph {
            forward,
            backward,
            shortcuts,
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
    }

    pub fn by_merging(graph: &ContractedGraph) -> HubGraph {
        let mut forward_labels = graph
            .upward_graph()
            .vertices()
            .map(|vertex| vec![HubLabelEntry::new(vertex)])
            .collect_vec();

        let mut backward_labels = graph
            .downward_graph()
            .vertices()
            .map(|vertex| vec![HubLabelEntry::new(vertex)])
            .collect_vec();

        for &vertex in
            graph
                .level_to_vertex()
                .iter()
                .rev()
                .progress_with(get_progressbar_long_jobs(
                    "Merging labels",
                    graph.level_to_vertex().len() as u64,
                ))
        {
            create_label(
                graph.upward_graph(),
                vertex,
                &mut forward_labels,
                &backward_labels,
            );
            create_label(
                graph.downward_graph(),
                vertex,
                &mut backward_labels,
                &forward_labels,
            );
        }

        forward_labels
            .iter_mut()
            .chain(backward_labels.iter_mut())
            .for_each(|label| set_predecessor(label));

        let forward = HalfHubGraph::new(&forward_labels);
        let backward = HalfHubGraph::new(&backward_labels);
        let shortcuts = graph.shortcuts().clone();

        HubGraph {
            forward,
            backward,
            shortcuts,
            level_to_vertex: graph.level_to_vertex().clone(),
            vertex_to_level: graph.vertex_to_level().clone(),
        }
    }

    pub fn average_label_size(&self) -> f32 {
        (self.forward.average_label_size() + self.backward.average_label_size()) / 2.0
    }
}

impl PathFinding for HubGraph {
    fn shortest_path(&self, source: Vertex, target: Vertex) -> Option<Path> {
        let forward_label = self.forward.get_label(source);
        let backward_label = self.backward.get_label(target);
        get_path_from_overlapp(forward_label, backward_label, &self.shortcuts)
    }

    fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        let forward_label = self.forward.get_label(source);
        let backward_label = self.backward.get_label(target);
        overlapp(forward_label, backward_label).map(|(distance, _)| distance)
    }
}

fn create_label(
    contracted_graph_direction1: &dyn Graph,
    vertex: u32,
    labels_direction1: &mut Vec<Vec<HubLabelEntry>>,
    labels_direction2: &Vec<Vec<HubLabelEntry>>,
) {
    let mut neighbor_labels = contracted_graph_direction1
        .edges(vertex)
        .map(|edge| {
            let neighbor_label = labels_direction1.get(edge.head as usize).unwrap();

            (Some(edge.clone()), neighbor_label)
        })
        .collect::<Vec<_>>();
    neighbor_labels.push((None, labels_direction1.get(vertex as usize).unwrap()));

    let mut forward_label = get_hub_label_by_merging(&neighbor_labels);
    prune_label(&mut forward_label, labels_direction2);
    labels_direction1[vertex as usize] = forward_label;
}

pub fn prune_label(
    label_direction1: &mut Vec<HubLabelEntry>,
    labels_direction2: &Vec<Vec<HubLabelEntry>>,
) {
    let mut new_label = label_direction1
        .par_iter()
        .filter(|entry| {
            let other_label = labels_direction2.get(entry.vertex as usize).unwrap();
            let true_distance = overlapp(label_direction1, other_label).unwrap().0;

            entry.distance == true_distance
        })
        .cloned()
        .collect::<Vec<_>>();

    std::mem::swap(&mut new_label, label_direction1);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubLabelEntry {
    pub vertex: Vertex,
    pub distance: Distance,
    /// index of predecessor. None if no predecessor.
    pub predecessor_index: Option<u32>,
}

impl HubLabelEntry {
    pub fn new(vertex: Vertex) -> Self {
        HubLabelEntry {
            vertex,
            distance: 0,
            predecessor_index: None,
        }
    }
}

pub fn get_path_from_overlapp(
    forward_label: &[HubLabelEntry],
    backward_label: &[HubLabelEntry],
    shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
) -> Option<Path> {
    let (distance, (forward_index, backward_index)) = overlapp(forward_label, backward_label)?;

    let mut forward_path = get_path_from_label(forward_label, forward_index);
    forward_path.pop();
    let mut backward_path = get_path_from_label(backward_label, backward_index);
    backward_path.reverse();

    forward_path.extend(backward_path);

    replace_shortcuts_slowly(&mut forward_path, shortcuts);

    Some(Path {
        vertices: forward_path,
        distance,
    })
}

pub fn get_path_from_label(label: &[HubLabelEntry], index: usize) -> Vec<Vertex> {
    let mut path = vec![label[index].vertex];

    let mut index = index;
    while let Some(predecessor_index) = label[index].predecessor_index {
        index = predecessor_index as usize;
        path.push(label[index].vertex);
    }

    path.reverse();
    path
}

pub fn overlapp(
    forward_label: &[HubLabelEntry],
    backward_label: &[HubLabelEntry],
) -> Option<(Distance, (usize, usize))> {
    let mut overlapp = None;

    let mut forward_index = 0;
    let mut backward_index = 0;

    while forward_index < forward_label.len() && backward_index < backward_label.len() {
        let forward_vertex = forward_label[forward_index].vertex;
        let backward_vertex = backward_label[backward_index].vertex;

        match forward_vertex.cmp(&backward_vertex) {
            Ordering::Less => {
                forward_index += 1;
            }
            Ordering::Equal => {
                let alternative_distance = forward_label[forward_index as usize].distance
                    + backward_label[backward_index as usize].distance;
                if alternative_distance
                    < overlapp
                        .map(|(current_distance, _)| current_distance)
                        .unwrap_or(Distance::MAX)
                {
                    overlapp = Some((alternative_distance, (forward_index, backward_index)));
                }

                forward_index += 1;
                backward_index += 1;
            }
            Ordering::Greater => {
                backward_index += 1;
            }
        }
    }

    overlapp
}

#[cfg(test)]
mod tests {
    use crate::{
        graphs::{large_test_graph, Graph},
        search::{
            ch::contracted_graph::ContractedGraph,
            hl::hub_graph::{get_path_from_overlapp, HubGraph},
        },
    };

    #[test]
    fn hub_graph_by_merging() {
        let (graph, tests) = large_test_graph();
        let contracted_graph = ContractedGraph::with_dijkstra_witness_search(&graph, u32::MAX);
        let hub_graph = HubGraph::by_merging(&contracted_graph);

        for test in tests {
            let forward_label = hub_graph.forward.get_label(test.source);
            let backward_label = hub_graph.backward.get_label(test.target);
            let path = get_path_from_overlapp(forward_label, backward_label, &hub_graph.shortcuts);

            let distance = path.as_ref().map(|path| path.distance);
            assert_eq!(test.distance, distance);

            let path_distance =
                path.and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
            assert_eq!(test.distance, path_distance)
        }
    }

    #[test]
    fn hub_graph_by_brute_force() {
        let (graph, tests) = large_test_graph();
        let contracted_graph = ContractedGraph::with_dijkstra_witness_search(&graph, u32::MAX);
        let hub_graph = HubGraph::by_brute_force(&graph, contracted_graph.vertex_to_level());

        for test in tests {
            let forward_label = hub_graph.forward.get_label(test.source);
            let backward_label = hub_graph.backward.get_label(test.target);
            let path = get_path_from_overlapp(forward_label, backward_label, &hub_graph.shortcuts);

            let distance = path.as_ref().map(|path| path.distance);
            assert_eq!(test.distance, distance);

            let path_distance =
                path.and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
            assert_eq!(test.distance, path_distance)
        }
    }
}
