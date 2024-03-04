use std::{
    collections::{BinaryHeap, HashMap},
    sync::atomic::{AtomicU32, Ordering},
};

use ahash::HashSet;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::graphs::{
    edge::DirectedWeightedEdge,
    graph::Graph,
    types::{VertexId, Weight},
};

use super::{binary_heap::MinimumItem, shortcut::Shortcut};

pub struct ContractionHelper<'a> {
    graph: &'a Graph,
    max_hops_in_witness_search: u32,
}

pub struct ShortcutSearchResult {
    pub shortcuts: Vec<Shortcut>,
    pub search_space_size: i32,
    pub edge_difference: i32,
}

impl<'a> ContractionHelper<'a> {
    pub fn new(graph: &'a Graph, max_hops_in_witness_search: u32) -> Self {
        Self {
            graph,
            max_hops_in_witness_search,
        }
    }

    /// Generates shortcuts for a node v.
    ///
    /// A shortcut (u, w) is generated if ((u, v), (v, w)) is the only shortest path between u and
    /// w.
    ///
    /// Returns a vector of (Edge, Vec<Edge>) where the first entry is the shortcut and the second
    /// entry the edges the shortcut replaces.
    pub fn get_shortcuts(&self, vertex: VertexId) -> ShortcutSearchResult {
        let uv_edges = &self.graph.in_edges(vertex);
        let vw_edges = &self.graph.out_edges(vertex);
        let max_vw_cost = vw_edges.iter().map(|edge| edge.weight).max().unwrap_or(0);

        let w_set: HashSet<VertexId> = vw_edges.iter().map(|edge| edge.head).collect();
        let search_space_size = AtomicU32::new(0);

        let shortcuts: Vec<_> = uv_edges
            .iter()
            .par_bridge()
            .flat_map(|uv_edge| {
                let mut shortcuts = Vec::new();
                // print!("{}", uv_edge.tail);

                let max_cost = uv_edge.weight + max_vw_cost;
                let witness_cost = self.witness_search(uv_edge.tail, vertex, max_cost, &w_set);
                search_space_size.fetch_add(witness_cost.len() as u32, Ordering::Relaxed);

                for vw_ede in vw_edges.iter() {
                    let uw_cost = uv_edge.weight + vw_ede.weight;
                    if &uw_cost < witness_cost.get(&vw_ede.head).unwrap_or(&u32::MAX) {
                        let edge = DirectedWeightedEdge {
                            tail: uv_edge.tail,
                            head: vw_ede.head,
                            weight: uw_cost,
                        };
                        let shortcut = Shortcut {
                            edge,
                            skiped_vertex: vertex,
                        };
                        shortcuts.push(shortcut);
                    }
                }
                shortcuts
            })
            .collect();

        let edge_difference = (shortcuts.len() - uv_edges.len() - vw_edges.len()) as i32;
        ShortcutSearchResult {
            shortcuts,
            search_space_size: search_space_size.into_inner() as i32,
            edge_difference,
        }
    }

    /// Generates shortcuts for a node v.
    ///
    /// A shortcut (u, w) is generated if ((u, v), (v, w)) is the only shortest path between u and
    /// w.
    ///
    /// Returns a vector of (Edge, Vec<Edge>) where the first entry is the shortcut and the second
    /// entry the edges the shortcut replaces.
    pub fn wittness_search_space(&self, vertex: VertexId) -> i32 {
        let uv_edges = &self.graph.in_edges(vertex);
        let vw_edges = &self.graph.out_edges(vertex);
        let max_vw_cost = vw_edges.iter().map(|edge| edge.weight).max().unwrap_or(0);

        let w_set: HashSet<VertexId> = vw_edges.iter().map(|edge| edge.head).collect();

        uv_edges
            .iter()
            .par_bridge()
            .map(|uv_edge| {
                let max_cost = uv_edge.weight + max_vw_cost;
                let witness_cost = self.witness_search(uv_edge.tail, vertex, max_cost, &w_set);

                witness_cost.len() as i32
            })
            .sum()
    }

    /// Performs a forward search from `source` node.
    ///
    /// Returns a `HashMap` where each entry consists of a node identifier (u32) and the associated cost (u32) to reach that node from the `source`.
    ///
    /// Parameters:
    /// - `source`: The starting node for the search.
    /// - `without`: A node identifier to be excluded from the search. The search will ignore paths through this node.
    /// - `max_cost`: The maximum allowable cost. Nodes that can only be reached with a cost higher than this value will not be included in the results.
    /// - `max_hops`: The maximum number of hops (edges traversed) allowed. Nodes that require more hops to reach than this limit will not be included in the results.
    ///
    /// Note: The search algorithm takes into account the cost and number of hops to reach each node. Nodes are included in the resulting map only if they meet the specified conditions regarding cost and hop count, and are not the `without` node.
    pub fn witness_search(
        &self,
        source: VertexId,
        without: VertexId,
        max_cost: u32,
        w_set: &HashSet<VertexId>,
    ) -> HashMap<VertexId, Weight> {
        let mut queue = BinaryHeap::new();
        let mut cost = HashMap::new();
        let mut hops = HashMap::new();

        let mut unseen_w = w_set.clone();

        queue.push(MinimumItem {
            weight: 0,
            vertex: source,
        });
        cost.insert(source, 0);
        hops.insert(source, 0);

        while let Some(MinimumItem { vertex, .. }) = queue.pop() {
            unseen_w.remove(&vertex);
            if unseen_w.is_empty() {
                break;
            }

            for edge in self.graph.out_edges(vertex).iter() {
                let alternative_cost = cost[&vertex] + edge.weight;
                let new_hops = hops[&vertex] + 1;
                if (edge.head != without)
                    && (alternative_cost <= max_cost)
                    && (new_hops <= self.max_hops_in_witness_search)
                {
                    let current_cost = *cost.get(&edge.head).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        queue.push(MinimumItem {
                            vertex: edge.head,
                            weight: alternative_cost,
                        });
                        cost.insert(edge.head, alternative_cost);
                        hops.insert(edge.head, new_hops);
                    }
                }
            }
        }

        cost
    }
}
