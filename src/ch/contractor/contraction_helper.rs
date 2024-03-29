use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
};

use ahash::HashSet;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    ch::ShortcutSearchResult,
    graphs::{
        edge::DirectedWeightedEdge,
        graph::Graph,
        {VertexId, Weight},
    },
    queue::{radix_queue::RadixQueue, DijkstaQueue, DijkstraQueueElement},
};

use super::Shortcut;

/// Generates shortcuts for a node v.
///
/// A shortcut (u, w) is generated if ((u, v), (v, w)) is the only shortest path between u and
/// w.
///
/// Returns a vector of (Edge, Vec<Edge>) where the first entry is the shortcut and the second
/// entry the edges the shortcut replaces.
pub fn get_shortcuts(graph: &Graph, vertex: VertexId, max_hops: u32) -> ShortcutSearchResult {
    let uv_edges = graph.in_edges(vertex);
    let vw_edges = graph.out_edges(vertex);
    let max_vw_cost = vw_edges.iter().map(|edge| edge.weight).max().unwrap_or(0);

    let w_set = vw_edges.iter().map(|edge| edge.head).collect();
    let search_space_size = AtomicU32::new(0);

    let shortcuts: Vec<_> = uv_edges
        .iter()
        .par_bridge()
        .flat_map(|uv_edge| {
            let mut shortcuts = Vec::new();

            let max_cost = uv_edge.weight + max_vw_cost;
            let witness_cost =
                witness_search(graph, uv_edge.tail, vertex, max_cost, max_hops, &w_set);
            search_space_size.fetch_add(witness_cost.len() as u32, Ordering::Relaxed);

            for vw_ede in vw_edges.iter() {
                let uw_cost = uv_edge.weight + vw_ede.weight;
                if &uw_cost < witness_cost.get(&vw_ede.head).unwrap_or(&u32::MAX) {
                    let edge = DirectedWeightedEdge {
                        tail: uv_edge.tail,
                        head: vw_ede.head,
                        weight: uw_cost,
                    };
                    let shortcut = Shortcut { edge, vertex };
                    shortcuts.push(shortcut);
                }
            }
            shortcuts
        })
        .collect();

    let edge_difference = (shortcuts.len() - uv_edges.len() - vw_edges.len()) as i32;
    let search_space_size = search_space_size.into_inner() as i32;
    ShortcutSearchResult {
        shortcuts,
        search_space_size,
        edge_difference,
    }
}

/// Performs a forward search from the specified `source` node, aiming to identify reachable nodes under certain constraints.
///
/// This method returns a `HashMap` where each entry maps a node identifier (`u32`) to the cost (`u32`) required to reach that node from the `source`. The search is constrained by maximum weight/cost, exclusion of a specific node, and a maximum number of hops.
///
/// Parameters:
/// - `source`: The identifier of the node from which the search starts.
/// - `without`: An identifier of a node to be excluded from the search. Paths through this node will be ignored.
/// - `max_weight`: The maximum allowable weight/cost for paths. Nodes reachable at a cost exceeding this limit will not be included in the results.
/// - `targets`: A reference to a `HashSet` of node identifiers representing target nodes. The search will attempt to reach these nodes within the specified constraints.
///
/// The function considers both the cost and the number of hops required to reach each node, including nodes in the result only if they can be reached without exceeding the `max_weight` and `max_hops` limits, and are not the `without` node.
///
/// Returns:
/// A `HashMap` mapping node identifiers to the cost of reaching them from the `source` node. This includes only the nodes that are reachable within the specified constraints and are not excluded by the `without` parameter.
///
/// The search employs a prioritized approach, leveraging a binary heap to efficiently explore paths based on their cumulative cost. It dynamically updates the cost and hops for each node as it discovers potentially more efficient paths. The algorithm terminates when all specified targets are reached or when it becomes impossible to find any more nodes meeting the criteria.
///
/// Note: This implementation assumes that `self.graph` provides access to the graph data structure, and `self.max_hops_in_witness_search` specifies the maximum number of hops allowed for the search.
pub fn witness_search(
    graph: &Graph,
    source: VertexId,
    without: VertexId,
    max_weight: Weight,
    max_hops: u32,
    targets: &HashSet<VertexId>,
) -> HashMap<VertexId, Weight> {
    let mut queue = RadixQueue::new();
    let mut weight = HashMap::new();
    let mut hops = HashMap::new();

    let mut targets = targets.clone();

    queue.push(DijkstraQueueElement::new(0, source));
    weight.insert(source, 0);
    hops.insert(source, 0);

    while let Some(DijkstraQueueElement { vertex, .. }) = queue.pop() {
        if targets.remove(&vertex) && targets.is_empty() {
            break;
        }

        for edge in graph.out_edges(vertex).iter() {
            let alternative_weight = weight[&vertex] + edge.weight;
            let alternative_hops = hops[&vertex] + 1;
            if (edge.head != without)
                && (alternative_weight <= max_weight)
                && (alternative_hops <= max_hops)
            {
                let current_cost = *weight.get(&edge.head).unwrap_or(&u32::MAX);
                if alternative_weight < current_cost {
                    queue.push(DijkstraQueueElement::new(alternative_weight, edge.head));
                    weight.insert(edge.head, alternative_weight);
                    hops.insert(edge.head, alternative_hops);
                }
            }
        }
    }

    weight
}
