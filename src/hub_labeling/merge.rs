use crate::{
    contraction_hierachy::ContractionHierarchy,
    data_structures::FlattenedNested,
    graph::{EdgeLike, GraphLike, compute_topological_layers},
    hub_labeling::{
        HubLabeling,
        entry::{LabelEntry, min_common_hub_distance},
    },
    types::{Distance, VertexId, distance_abs_diff_eq},
};

use rayon::prelude::*;

use indicatif::ProgressBar;
use rustc_hash::FxHashMap;

/// Builds a HubLabeling from a ContractionHierarchy by merging.
///
/// The up and down labels are computed layer by layer in a shared topological
/// order of both up and down graphs. Within one layer, vertices are independent
/// and can be processed in parallel.
///
/// Returns `None` if the combined up down graph is cyclic and no
/// topological layering exists.
pub(super) fn merge<D: Distance>(
    contraction_hierarchy: &ContractionHierarchy<D>,
    epsilon: D,
) -> Option<HubLabeling<D>> {
    let up_graph = contraction_hierarchy.up_graph();
    let down_graph = contraction_hierarchy.down_graph();
    let topological_layers = compute_topological_layers(&[up_graph, down_graph])?;

    let num_vertices = contraction_hierarchy.num_vertices();
    let mut up_labels = initialize_labels(num_vertices);
    let mut down_labels = initialize_labels(num_vertices);

    let bar = ProgressBar::new(num_vertices as u64);
    for vertices in topological_layers {
        // Build labels in parallel.
        let labels = vertices
            .par_iter()
            .map(|&vertex| {
                let up_edges_vertex = up_graph.outgoing_edges(vertex);
                let mut up_label = merge_label(up_edges_vertex, &up_labels, vertex);
                up_label = prune_label(&down_labels, &up_label, epsilon);
                up_label.shrink_to_fit();

                let down_edges_vertex = down_graph.outgoing_edges(vertex);
                let mut down_label = merge_label(down_edges_vertex, &down_labels, vertex);
                down_label = prune_label(&up_labels, &down_label, epsilon);
                down_label.shrink_to_fit();

                bar.inc(1);

                (vertex, up_label, down_label)
            })
            .collect::<Vec<_>>();

        // Assign them sequential.
        labels
            .into_iter()
            .for_each(|(vertex, up_label, down_label)| {
                up_labels[vertex.as_usize()] = up_label;
                down_labels[vertex.as_usize()] = down_label;
            });
    }

    bar.finish();

    Some(HubLabeling::new(
        FlattenedNested::from_nested(&up_labels),
        FlattenedNested::from_nested(&down_labels),
    ))
}

/// Creates one initial label for every vertex.
///
/// Each label contains only its own vertex as hub, at distance zero and without
/// a predecessor.
fn initialize_labels<D: Distance>(num_vertices: usize) -> Vec<Vec<LabelEntry<D>>> {
    (0..num_vertices)
        .map(|vertex| {
            let label_entry = LabelEntry {
                hub: VertexId::new(vertex as u32),
                distance: D::zero(),
                predecessor_hub: None,
            };
            vec![label_entry]
        })
        .collect()
}

/// Builds the unpruned label for `vertex` from already computed neighbor labels.
///
/// Each outgoing edge extends every label entry of its head vertex by the edge
/// weight. If several outgoing edges produce an entry for the same hub, only the
/// candidate with the smallest distance is kept. The resulting label is sorted
/// by hub.
fn merge_label<D, E>(
    edges: &[E],
    labels: &[Vec<LabelEntry<D>>],
    vertex: VertexId,
) -> Vec<LabelEntry<D>>
where
    D: Distance,
    E: EdgeLike<Weight = D>,
{
    let mut entries = FxHashMap::default();
    entries.insert(vertex, (D::zero(), None));

    for edge in edges {
        for entry in &labels[edge.head().as_usize()] {
            let candidate_distance = entry.distance + edge.weight();
            let candidate_predecessor_hub = Some(entry.predecessor_hub.unwrap_or(vertex));

            entries
                .entry(entry.hub)
                .and_modify(|(best_distance, best_predecessor_hub)| {
                    if candidate_distance < *best_distance {
                        *best_distance = candidate_distance;
                        *best_predecessor_hub = candidate_predecessor_hub;
                    }
                })
                .or_insert((candidate_distance, candidate_predecessor_hub));
        }
    }

    let mut label = entries
        .into_iter()
        .map(|(hub, (distance, predecessor_hub))| LabelEntry {
            hub,
            distance,
            predecessor_hub,
        })
        .collect::<Vec<_>>();

    label.sort_unstable_by_key(|entry| entry.hub);
    label
}

/// Removes label entries whose stored distance is not the true shortest distance.
///
/// An entry is kept only if its distance matches the shortest distance obtained
/// through the opposite labels. Entries with larger distances cannot contribute
/// to a shortest-path query.
///
/// Expects `dir1_label` and each relevant label in `dir2_labels` to have at
/// least one common hub, so the intersection lookup always succeeds.
fn prune_label<D: Distance>(
    dir2_labels: &[Vec<LabelEntry<D>>],
    dir1_label: &[LabelEntry<D>],
    epsilon: D,
) -> Vec<LabelEntry<D>> {
    dir1_label
        .iter()
        .filter(|entry| {
            let dir2_label = &dir2_labels[entry.hub.as_usize()];
            let (true_distance, _dir1_index, _dir2_index) =
                min_common_hub_distance(dir1_label, dir2_label).unwrap();

            distance_abs_diff_eq(entry.distance, true_distance, epsilon)
        })
        .copied()
        .collect()
}
