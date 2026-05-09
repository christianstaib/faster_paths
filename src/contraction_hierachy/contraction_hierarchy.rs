use crate::{
    contraction_hierachy::edge::ContractionEdge,
    graph::{FastGraph, GraphLike},
    types::{Distance, VertexId},
};

pub struct ContractionHierarchy<D: Distance> {
    up_graph: FastGraph<ContractionEdge<D>>,
    down_graph: FastGraph<ContractionEdge<D>>,
}

impl<D: Distance> ContractionHierarchy<D> {
    pub fn new(
        up_graph: FastGraph<ContractionEdge<D>>,
        down_graph: FastGraph<ContractionEdge<D>>,
    ) -> Self {
        Self {
            up_graph,
            down_graph,
        }
    }

    pub fn up_graph(&self) -> &FastGraph<ContractionEdge<D>> {
        &self.up_graph
    }

    pub fn down_graph(&self) -> &FastGraph<ContractionEdge<D>> {
        &self.down_graph
    }

    pub fn num_vertices(&self) -> usize {
        std::cmp::max(self.up_graph.num_vertices(), self.down_graph.num_vertices())
    }
}

pub fn extract_contraction_order<D: Distance>(ch: &ContractionHierarchy<D>) -> Vec<Vec<VertexId>> {
    let num_vertices = ch.num_vertices();
    let mut indegrees = vec![0; num_vertices];

    for edge in ch.up_graph().edges().chain(ch.down_graph().edges()) {
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
        let mut layer = Vec::with_capacity(current_layer.len());
        let mut next_layer = Vec::new();

        while let Some(vertex) = current_layer.pop() {
            layer.push(vertex);
            visited += 1;

            for edge in ch
                .up_graph()
                .out_edges(vertex)
                .iter()
                .chain(ch.down_graph().out_edges(vertex))
            {
                let head_indegree = &mut indegrees[edge.head.as_usize()];
                *head_indegree -= 1;

                if *head_indegree == 0 {
                    next_layer.push(edge.head);
                }
            }
        }

        layers.push(layer);
        current_layer = next_layer;
    }

    assert_eq!(
        visited, num_vertices,
        "contraction hierarchy must be acyclic"
    );

    layers.reverse();
    layers
}
