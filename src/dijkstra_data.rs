use std::usize;

use crate::{
    graphs::{
        path::Path,
        types::{VertexId, Weight},
    },
    queue::heap_queue::{HeapQueue, State},
};

#[derive(Clone)]
pub struct DijsktraEntry {
    pub predecessor: Option<VertexId>,
    pub weight: Option<Weight>,
    pub is_expanded: bool,
}

impl DijsktraEntry {
    fn new() -> DijsktraEntry {
        DijsktraEntry {
            predecessor: None,
            weight: None,
            is_expanded: false,
        }
    }
}

#[derive(Clone)]
pub struct DijkstraData {
    pub queue: HeapQueue,
    pub verticies: Vec<DijsktraEntry>,
}

impl DijkstraData {
    pub fn new(num_nodes: usize, source: VertexId) -> DijkstraData {
        let mut queue = HeapQueue::new();
        let mut nodes = vec![DijsktraEntry::new(); num_nodes];
        nodes[source as usize].weight = Some(0);
        queue.insert(0, source);
        DijkstraData {
            queue,
            verticies: nodes,
        }
    }

    pub fn search_space_size(&self) -> u32 {
        self.verticies
            .iter()
            .filter(|entry| entry.is_expanded)
            .count() as u32
    }

    pub fn pop(&mut self) -> Option<State> {
        while let Some(state) = self.queue.pop() {
            if !self.verticies[state.value as usize].is_expanded {
                self.verticies[state.value as usize].is_expanded = true;
                return Some(state);
            }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn update(&mut self, source: VertexId, target: VertexId, edge_weight: Weight) {
        let alternative_cost = self.verticies[source as usize].weight.unwrap() + edge_weight;
        let current_cost = self.verticies[target as usize].weight.unwrap_or(u32::MAX);
        if alternative_cost < current_cost {
            self.verticies[target as usize].predecessor = Some(source);
            self.verticies[target as usize].weight = Some(alternative_cost);
            self.queue.insert(alternative_cost, target);
        }
    }

    pub fn get_route(&self, target: VertexId) -> Option<Path> {
        let mut route = vec![target];
        let mut current = target;
        while let Some(predecessor) = self.verticies[current as usize].predecessor {
            current = predecessor;
            route.push(current);
        }
        route.reverse();
        Some(Path {
            weight: self.verticies[target as usize].weight?,
            vertices: route,
        })
    }

    pub fn get_scanned_points(&self) -> Vec<usize> {
        self.verticies
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.weight.is_some())
            .map(|(i, _)| i)
            .collect()
    }
}
