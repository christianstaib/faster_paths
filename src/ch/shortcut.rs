use crate::{
    ch::{contraction_hierarchy::ContractionHierarchy, edge::Edge},
    flattened_nested::FlattenedNested,
    types::VertexId,
};

/// Finds an edge from `tail` to `head` in `graph`.
///
/// If multiple such edges exist, an arbitrary matching edge may be returned.
/// Returns `None` if no matching edge exists.
///
/// Assumes that the outgoing edges of `tail` are sorted by their head vertex,
/// since the lookup is performed using binary search.
fn find_edge(graph: &FlattenedNested<Edge>, tail: VertexId, head: VertexId) -> Option<&Edge> {
    let edges = graph.nested(tail.as_usize());

    edges
        .binary_search_by_key(&head, |edge| edge.head())
        .ok()
        .map(|idx| &edges[idx])
}

/// Reconstructs the full shortest path ending at `meeting_vertex`.
///
/// Retrieves and unpacks both the upward and downward search paths,
/// expanding any shortcut edges back into their constituent vertices.
/// Returns `None` if `meeting_vertex` is unreachable in either search states or the unpacking
/// of the shortcuts failed due to an incorrect contraction hierarchy.
pub fn unpack_and_concat_shortcut_paths(
    contraction_hierarchy: &ContractionHierarchy,
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
/// The input path is interpreted in `dir1_graph`. Shortcut edges are unpacked
/// iteratively using both graph directions. Returns `None` if an expected edge
/// is not found or the input is empty.
pub fn unpack_shortcuts(
    dir1_graph: &FlattenedNested<Edge>,
    dir2_graph: &FlattenedNested<Edge>,
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
        let Some(skipped) = edge.skipped() else {
            // Original edges contribute one vertex to the unpacked path.
            path.push(match dir {
                Dir::Dir1 => edge.tail(),
                Dir::Dir2 => edge.head(),
            });
            continue;
        };

        let (dir1_child_head, dir2_child_head) = match dir {
            Dir::Dir1 => (edge.head(), edge.tail()),
            Dir::Dir2 => (edge.tail(), edge.head()),
        };

        // Push in reverse order because the stack is processed LIFO.
        stack.push((find_edge(dir2_graph, skipped, dir2_child_head)?, Dir::Dir2));
        stack.push((find_edge(dir1_graph, skipped, dir1_child_head)?, Dir::Dir1));
    }

    Some(path)
}
