use crate::graph::GraphLike;
use crate::{
    contraction_hierachy::{contraction_hierarchy::ContractionHierarchy, edge::ContractionEdge},
    graph::FastGraph,
    types::{Distance, VertexId},
};

/// Finds an edge from `tail` to `head` in `graph`.
///
/// If multiple matching edges exist, an arbitrary one may be returned.
/// Returns `None` if no such edge exists.
///
/// The outgoing edges of `tail` must be sorted by their head vertex, because the
/// lookup is performed using binary search.
fn find_edge<D: Distance>(
    graph: &FastGraph<ContractionEdge<D>>,
    tail: VertexId,
    head: VertexId,
) -> Option<&ContractionEdge<D>> {
    let edges = graph.out_edges(tail);

    edges
        .binary_search_by_key(&head, |edge| edge.head)
        .ok()
        .map(|idx| &edges[idx])
}

/// Unpacks the upward and downward shortcut paths and concatenates them.
///
/// Both input paths are expected to be reversed shortcut paths starting at the
/// meeting vertex.
///
/// Returns the fully unpacked vertex path from source to target, or `None` if
/// one of the shortcut paths is empty, an expected edge is missing, or a shortcut
/// cannot be unpacked with the given contraction hierarchy.
pub fn unpack_and_concat_shortcut_paths<D: Distance>(
    contraction_hierarchy: &ContractionHierarchy<D>,
    up_reversed_shortcut_path: &Vec<VertexId>,
    down_reversed_shortcut_path: &Vec<VertexId>,
) -> Option<Vec<VertexId>> {
    let up_path = unpack_shortcuts(
        contraction_hierarchy.up_graph(),
        contraction_hierarchy.down_graph(),
        &up_reversed_shortcut_path,
    )?;

    let down_path = unpack_shortcuts(
        contraction_hierarchy.down_graph(),
        contraction_hierarchy.up_graph(),
        &down_reversed_shortcut_path,
    )?;

    Some(
        up_path
            .into_iter()
            .rev()
            .chain(down_path.into_iter().skip(1))
            .collect(),
    )
}

/// Expands a reversed shortcut path into the corresponding non-reversed vertex path.
///
/// Shortcut edges are unpacked iteratively using both graphs. Returns `None` if
/// an expected edge is not found or the input is empty.
pub fn unpack_shortcuts<D: Distance>(
    dir1_graph: &FastGraph<ContractionEdge<D>>,
    dir2_graph: &FastGraph<ContractionEdge<D>>,
    dir1_reversed_shortcut_path: &[VertexId],
) -> Option<Vec<VertexId>> {
    enum Dir {
        Dir1,
        Dir2,
    }

    // Seed the stack with the shortcut edges of the reversed input path.
    let mut stack = Vec::new();
    for pair in dir1_reversed_shortcut_path.windows(2).rev() {
        stack.push((find_edge(dir1_graph, pair[1], pair[0])?, Dir::Dir1));
    }

    let mut path = vec![*dir1_reversed_shortcut_path.first()?];
    while let Some((edge, dir)) = stack.pop() {
        let Some(skipped) = edge.skipped else {
            // Original edges contribute one vertex to the unpacked path.
            path.push(match dir {
                Dir::Dir1 => edge.tail,
                Dir::Dir2 => edge.head,
            });
            continue;
        };

        let (dir1_child_head, dir2_child_head) = match dir {
            Dir::Dir1 => (edge.head, edge.tail),
            Dir::Dir2 => (edge.tail, edge.head),
        };

        // Push in reverse order because the stack is processed LIFO.
        stack.push((find_edge(dir2_graph, skipped, dir2_child_head)?, Dir::Dir2));
        stack.push((find_edge(dir1_graph, skipped, dir1_child_head)?, Dir::Dir1));
    }

    Some(path)
}
