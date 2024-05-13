use ahash::{HashMap, HashMapExt};

use super::{dijkstra_data_vec::DijsktraEntry, DijkstraData};
use crate::{
    graphs::{path::Path, VertexId, Weight},
    queue::{radix_queue::RadixQueue, DijkstaQueue, DijkstraQueueElement},
};

pub struct DijkstraDataHashMap {
    pub queue: Box<dyn DijkstaQueue>,
    pub vertices: HashMap<VertexId, DijsktraEntry>,
}

impl DijkstraDataHashMap {
    pub fn new(_num_nodes: usize, source: VertexId) -> DijkstraDataHashMap {
        let queue = Box::new(RadixQueue::new());
        let vertices = HashMap::new();
        let mut data = DijkstraDataHashMap { queue, vertices };

        data.vertices.entry(source).or_default().weight = Some(0);
        data.queue.push(DijkstraQueueElement::new(0, source));

        data
    }

    pub fn clear(&mut self, source: VertexId) {
        self.queue.clear();
        self.vertices.clear();

        self.vertices.entry(source).or_default().weight = Some(0);
        self.queue.push(DijkstraQueueElement::new(0, source));
    }
}

impl DijkstraData for DijkstraDataHashMap {
    fn search_space_size(&self) -> u32 {
        //     self.vertices
        //         .iter()
        //         .filter(|entry| entry.is_expanded)
        //         .count() as u32
        0
    }

    fn pop(&mut self) -> Option<DijkstraQueueElement> {
        while let Some(state) = self.queue.pop() {
            let entry = self.vertices.get_mut(&state.vertex).unwrap();
            if !entry.is_expanded {
                entry.is_expanded = true;
                return Some(state);
            }
        }

        None
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn update(&mut self, tail: VertexId, head: VertexId, edge_weight: Weight) {
        let alternative_cost = self.vertices[&tail].weight.unwrap() + edge_weight;
        let head_entry = self.vertices.entry(head).or_default();
        let current_cost = head_entry.weight.unwrap_or(u32::MAX);
        if alternative_cost < current_cost {
            head_entry.predecessor = Some(tail);
            head_entry.weight = Some(alternative_cost);
            self.queue
                .push(DijkstraQueueElement::new(alternative_cost, head));
        }
    }

    fn get_path(&self, target: VertexId) -> Option<Path> {
        let mut route = vec![target];
        let mut current = target;
        while let Some(predecessor) = self.vertices.get(&current)?.predecessor {
            current = predecessor;
            route.push(current);
        }
        route.reverse();
        Some(Path {
            weight: self.vertices[&target].weight?,
            vertices: route,
        })
    }

    fn dijkstra_rank(&self) -> u32 {
        self.vertices
            .values()
            .filter(|entry| entry.is_expanded)
            .count() as u32
    }

    fn get_scanned_vertices(&self) -> Vec<VertexId> {
        // self.vertices
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, entry)| entry.weight.is_some())
        //     .map(|(i, _)| i as VertexId)
        //     .collect()
        Vec::new()
    }

    fn get_vertex_entry(&mut self, vertex: VertexId) -> &mut DijsktraEntry {
        self.vertices.entry(vertex).or_default()
    }
}
