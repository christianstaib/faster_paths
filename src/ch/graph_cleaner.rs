use std::collections::HashMap;

use crate::graph::Graph;

pub fn remove_edge_to_self(graph: &mut Graph) {
    for i in 0..graph.out_edges.len() {
        graph.out_edges[i].retain(|edge| edge.head != i as u32);
    }

    for i in 0..graph.in_edges.len() {
        graph.in_edges[i].retain(|edge| edge.tail != i as u32);
    }
}

pub fn removing_double_edges(graph: &mut Graph) {
    for tail in 0..graph.out_edges.len() {
        let mut edge_map = HashMap::new();
        for edge in &graph.out_edges[tail] {
            let edge_tuple = (edge.head, tail);
            let current_cost = edge_map.get(&edge_tuple).unwrap_or(&u32::MAX);
            if &edge.cost < current_cost {
                edge_map.insert(edge_tuple, edge.cost);
            }
        }
        graph.out_edges[tail]
            .retain(|edge| edge.cost <= *edge_map.get(&(edge.head, tail)).unwrap());
    }

    for head in 0..graph.in_edges.len() {
        let mut edge_map = HashMap::new();
        for edge in &graph.in_edges[head] {
            let edge_tuple = (head, edge.tail);
            let current_cost = edge_map.get(&edge_tuple).unwrap_or(&u32::MAX);
            if &edge.cost < current_cost {
                edge_map.insert(edge_tuple, edge.cost);
            }
        }
        graph.in_edges[head].retain(|edge| edge.cost <= *edge_map.get(&(head, edge.tail)).unwrap());
    }
}
