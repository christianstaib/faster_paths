use std::cmp::Ordering;

use super::half_hub_graph::HalfHubGraph;
use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex},
    search::ch::contracted_graph::ContractedGraph,
};

pub struct HubGraph {
    pub forward: HalfHubGraph,
    pub backward: HalfHubGraph,
}

impl HubGraph {
    pub fn by_brute_force<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        vertex_to_level: &Vec<u32>,
    ) -> HubGraph {
        let forward = HalfHubGraph::by_brute_force(graph.out_graph(), vertex_to_level);
        let backward = HalfHubGraph::by_brute_force(graph.in_graph(), vertex_to_level);

        HubGraph { forward, backward }
    }

    pub fn by_merging(contracted_graph: &ContractedGraph) -> HubGraph {
        let forward = HalfHubGraph::by_merging(
            &contracted_graph.upward_graph,
            &contracted_graph.level_to_vertex,
        );
        let backward = HalfHubGraph::by_merging(
            &contracted_graph.downward_graph,
            &contracted_graph.level_to_vertex,
        );

        HubGraph { forward, backward }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubLabelEntry {
    pub vertex: Vertex,
    pub distance: Distance,
    /// relative index of predecessor. Zero if no predecessor.
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

pub fn overlapp(
    forward_label: &[HubLabelEntry],
    backward_label: &[HubLabelEntry],
) -> Option<(Distance, (usize, usize))> {
    let mut overlapp = None;

    let mut forward_iter = forward_label
        .iter()
        .enumerate()
        .map(|(index, entry)| (index, entry.vertex))
        .peekable();
    let mut backward_iter = backward_label
        .iter()
        .enumerate()
        .map(|(index, entry)| (index, entry.vertex))
        .peekable();

    while let (Some(&(forward_index, forward_vertex)), Some(&(backward_index, backward_vertex))) =
        (forward_iter.peek(), backward_iter.peek())
    {
        match forward_vertex.cmp(&backward_vertex) {
            Ordering::Less => {
                forward_iter.next();
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

                forward_iter.next();
                backward_iter.next();
            }
            Ordering::Greater => {
                backward_iter.next();
            }
        }
    }

    overlapp
}
