use crate::{
    contraction_hierachy::edge::ContractionEdge,
    graph::{CsrGraph, GraphLike},
    types::{Distance, VertexId},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ContractionHierarchy<D: Distance> {
    up_graph: CsrGraph<ContractionEdge<D>>,
    down_graph: CsrGraph<ContractionEdge<D>>,
}

impl<D: Distance> ContractionHierarchy<D> {
    pub fn new(
        up_graph: CsrGraph<ContractionEdge<D>>,
        down_graph: CsrGraph<ContractionEdge<D>>,
    ) -> Self {
        Self {
            up_graph,
            down_graph,
        }
    }

    pub fn up_graph(&self) -> &CsrGraph<ContractionEdge<D>> {
        &self.up_graph
    }

    pub fn down_graph(&self) -> &CsrGraph<ContractionEdge<D>> {
        &self.down_graph
    }

    pub fn num_vertices(&self) -> usize {
        std::cmp::max(self.up_graph.num_vertices(), self.down_graph.num_vertices())
    }
}

pub fn extract_contraction_order<D: Distance>(
    ch: &ContractionHierarchy<D>,
) -> Option<Vec<Vec<VertexId>>> {
    let num_vertices = ch.num_vertices();
    let mut indegrees = vec![0; num_vertices];

    let all_up_edges = ch.up_graph.edges();
    let all_down_edges = ch.down_graph.edges();
    for edge in all_up_edges.chain(all_down_edges) {
        indegrees[edge.head.as_usize()] += 1;
    }

    let mut current_layer = indegrees
        .iter()
        .enumerate()
        .filter_map(|(vertex, &indegree)| (indegree == 0).then_some(VertexId::new(vertex as u32)))
        .collect::<Vec<_>>();

    let mut layers = Vec::new();
    let mut visited = 0;

    while !current_layer.is_empty() {
        let mut next_layer = Vec::new();

        for vertex in current_layer.iter().copied() {
            visited += 1;

            let up_edges = ch.up_graph.out_edges(vertex).iter();
            let down_edges = ch.down_graph.out_edges(vertex).iter();
            for edge in up_edges.chain(down_edges) {
                let head_indegree = &mut indegrees[edge.head.as_usize()];
                *head_indegree -= 1;

                if *head_indegree == 0 {
                    next_layer.push(edge.head);
                }
            }
        }

        layers.push(current_layer);
        current_layer = next_layer;
    }

    // contraction hierarchy must be acyclic
    if visited != num_vertices {
        return None;
    }

    layers.reverse();
    Some(layers)
}
