use std::collections::{BinaryHeap, HashMap};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::graphs::{edge::DirectedWeightedEdge, graph::Graph, types::VertexId};

use super::binary_heap::MinimumItem;

pub struct ContractionHelper<'a> {
    graph: &'a Graph,
    max_hops_in_witness_search: u32,
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
    pub fn generate_shortcuts(&self, v: u32) -> Vec<(DirectedWeightedEdge, VertexId)> {
        let uv_edges = &self.graph.in_edges[v as usize];
        let vw_edges = &self.graph.out_edges[v as usize];
        let max_vw_cost = vw_edges.iter().map(|edge| edge.cost).max().unwrap_or(0);

        uv_edges
            .iter()
            .par_bridge()
            .flat_map(|uv_edge| {
                let mut shortcuts = Vec::new();

                let max_cost = uv_edge.cost + max_vw_cost;
                let witness_cost = self.witness_search(uv_edge.tail, v, max_cost);

                for vw_ede in vw_edges.iter() {
                    let uw_cost = uv_edge.cost + vw_ede.cost;
                    if &uw_cost < witness_cost.get(&vw_ede.head).unwrap_or(&u32::MAX) {
                        let edge = DirectedWeightedEdge {
                            tail: uv_edge.tail,
                            head: vw_ede.head,
                            weight: uw_cost,
                        };
                        shortcuts.push((edge, v));
                    }
                }
                shortcuts
            })
            .collect()
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
    ) -> HashMap<u32, u32> {
        let mut queue = BinaryHeap::new();
        let mut cost = HashMap::new();
        let mut hops = HashMap::new();

        queue.push(MinimumItem {
            weight: 0,
            vertex: source,
        });
        cost.insert(source, 0);
        hops.insert(source, 0);

        while let Some(MinimumItem { vertex: node, .. }) = queue.pop() {
            for edge in &self.graph.out_edges[node as usize] {
                let alternative_cost = cost[&node] + edge.cost;
                let new_hops = hops[&node] + 1;
                if (edge.head != without)
                    && (alternative_cost <= max_cost)
                    && (new_hops <= self.max_hops_in_witness_search)
                {
                    let current_cost = *cost.get(&edge.head).unwrap_or(&u32::MAX);
                    if alternative_cost < current_cost {
                        queue.push(MinimumItem {
                            weight: alternative_cost,
                            vertex: edge.head,
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
