use crate::{
    graphs::{path::Path, VertexId, Weight},
    queue::DijkstraQueueElement,
};

use self::dijkstra_data_vec::DijsktraEntry;

pub mod dijkstra_data_map;
pub mod dijkstra_data_vec;

pub trait DijkstraData {
    fn search_space_size(&self) -> u32;

    fn pop(&mut self) -> Option<DijkstraQueueElement>;

    fn is_empty(&self) -> bool;

    fn update(&mut self, tail: VertexId, head: VertexId, edge_weight: Weight);

    fn get_path(&self, target: VertexId) -> Option<Path>;

    fn dijkstra_rank(&self) -> u32;

    fn get_scanned_vertices(&self) -> Vec<VertexId>;

    fn get_vertex_entry(&mut self, vertex: VertexId) -> &mut DijsktraEntry;
}
