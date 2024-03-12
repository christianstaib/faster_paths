use crate::{
    graphs::{
        path::Path,
        types::{VertexId, Weight},
    },
    queue::{heap_queue::HeapQueue, State},
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
        queue.push(State::new(0, source));
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
            if !self.verticies[state.vertex as usize].is_expanded {
                self.verticies[state.vertex as usize].is_expanded = true;
                return Some(state);
            }
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn update(&mut self, tail: VertexId, head: VertexId, edge_weight: Weight) {
        let alternative_cost = self.verticies[tail as usize].weight.unwrap() + edge_weight;
        let current_cost = self.verticies[head as usize].weight.unwrap_or(u32::MAX);
        if alternative_cost < current_cost {
            self.verticies[head as usize].predecessor = Some(tail);
            self.verticies[head as usize].weight = Some(alternative_cost);
            self.queue.push(State::new(alternative_cost, head));
        }
    }

    pub fn get_path(&self, target: VertexId) -> Option<Path> {
        let mut route = vec![target];
        let mut current = target;
        while let Some(predecessor) = self.verticies.get(current as usize)?.predecessor {
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
