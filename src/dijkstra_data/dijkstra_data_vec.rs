use std::usize;

use super::DijkstraData;
use crate::{
    graphs::{path::Path, VertexId, Weight},
    queue::{radix_queue::RadixQueue, DijkstaQueue, DijkstraQueueElement},
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

impl Default for DijsktraEntry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DijkstraDataVec {
    pub queue: Box<dyn DijkstaQueue>,
    pub vertices: Vec<DijsktraEntry>,
}

impl DijkstraDataVec {
    pub fn new(num_nodes: usize, source: VertexId) -> DijkstraDataVec {
        let queue = Box::new(RadixQueue::new());
        let vertices = vec![DijsktraEntry::new(); num_nodes];
        let mut data = DijkstraDataVec { queue, vertices };

        data.vertices[source as usize].weight = Some(0);
        data.queue.push(DijkstraQueueElement::new(0, source));

        data
    }
}

impl DijkstraData for DijkstraDataVec {
    fn search_space_size(&self) -> u32 {
        self.vertices
            .iter()
            .filter(|entry| entry.is_expanded)
            .count() as u32
    }

    fn pop(&mut self) -> Option<DijkstraQueueElement> {
        while let Some(state) = self.queue.pop() {
            if !self.vertices[state.vertex as usize].is_expanded {
                self.vertices[state.vertex as usize].is_expanded = true;
                return Some(state);
            }
        }

        None
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn update(&mut self, tail: VertexId, head: VertexId, edge_weight: Weight) {
        let alternative_cost = self.vertices[tail as usize].weight.unwrap() + edge_weight;
        let current_cost = self.vertices[head as usize].weight.unwrap_or(u32::MAX);
        if alternative_cost < current_cost {
            self.vertices[head as usize].predecessor = Some(tail);
            self.vertices[head as usize].weight = Some(alternative_cost);
            self.queue
                .push(DijkstraQueueElement::new(alternative_cost, head));
        }
    }

    fn get_path(&self, target: VertexId) -> Option<Path> {
        let mut route = vec![target];
        let mut current = target;
        while let Some(predecessor) = self.vertices.get(current as usize)?.predecessor {
            current = predecessor;
            route.push(current);
        }
        route.reverse();
        Some(Path {
            weight: self.vertices[target as usize].weight?,
            vertices: route,
        })
    }

    fn dijkstra_rank(&self) -> u32 {
        self.vertices
            .iter()
            .filter(|entry| entry.is_expanded)
            .count() as u32
    }

    fn get_scanned_vertices(&self) -> Vec<VertexId> {
        self.vertices
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.weight.is_some())
            .map(|(i, _)| i as VertexId)
            .collect()
    }

    fn get_vertex_entry(&mut self, vertex: VertexId) -> &mut DijsktraEntry {
        &mut self.vertices[vertex as usize]
    }
}
