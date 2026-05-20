use crate::{
    graph::{EdgeLike, GraphLike},
    types::VertexId,
};

/// Computes *one* layered topological sorting that is valid for *multiple* graphs at once, if it
/// exists.
///
/// Kahn's algorithm is executed on the union of all directed edges from the
/// given graphs. If the combined graph is acyclic, the topological layers are
/// returned. If it contains a cycle, `None` is returned.
///
/// The returned layers are ordered from top to bottom: vertices in layer `0`
/// are at the head/top end of directed paths. Therefore, for an edge
/// `tail -> head`, `head` appears in an earlier layer than `tail`.
pub fn compute_topological_layers<G: GraphLike>(graphs: &[&G]) -> Option<Vec<Vec<VertexId>>> {
    let num_vertices = graphs
        .iter()
        .map(|graph| graph.num_vertices())
        .max()
        .unwrap_or_default();
    let mut indegrees = vec![0; num_vertices];

    for edge in graphs.iter().flat_map(|graph| graph.all_edges()) {
        indegrees[edge.head().as_usize()] += 1;
    }

    let mut current_layer = indegrees
        .iter()
        .enumerate()
        .filter_map(|(vertex, &indegree)| (indegree == 0).then_some(VertexId::new(vertex as u32)))
        .collect::<Vec<_>>();

    let mut layers = Vec::new();
    let mut visited = 0;

    while !current_layer.is_empty() {
        visited += current_layer.len();
        let mut next_layer = Vec::new();

        for &vertex in &current_layer {
            for edge in graphs.iter().flat_map(|graph| graph.outgoing_edges(vertex)) {
                let head_indegree = &mut indegrees[edge.head().as_usize()];
                *head_indegree -= 1;

                if *head_indegree == 0 {
                    next_layer.push(edge.head());
                }
            }
        }

        layers.push(current_layer);
        current_layer = next_layer;
    }

    (visited == num_vertices).then(|| {
        layers.reverse();
        layers
    })
}
