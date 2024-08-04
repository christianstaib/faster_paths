use crate::graphs::{Graph, VertexId, Weight};

pub trait DijkstraData {
    fn clear(&mut self);

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId>;

    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId);

    fn get_distance(&self, vertex: VertexId) -> Option<VertexId>;

    fn set_distance(&mut self, vertex: VertexId, distance: Weight);
}

pub struct DijkstraDataVec {
    predecessors: Vec<VertexId>,
    distances: Vec<Weight>,
}

impl DijkstraDataVec {
    pub fn new(graph: &dyn Graph) -> Self {
        DijkstraDataVec {
            predecessors: vec![VertexId::MAX; graph.number_of_vertices() as usize],
            distances: vec![Weight::MAX; graph.number_of_vertices() as usize],
        }
    }
}

impl DijkstraData for DijkstraDataVec {
    fn clear(&mut self) {
        for vertex in 0..self.predecessors.len() {
            self.predecessors[vertex] = VertexId::MAX;
            self.distances[vertex] = Weight::MAX;
        }
    }

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId> {
        if self.predecessors[vertex as usize] != VertexId::MAX {
            return Some(self.predecessors[vertex as usize]);
        }

        None
    }

    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId) {
        self.predecessors[vertex as usize] = predecessor;
    }

    fn get_distance(&self, vertex: VertexId) -> Option<Weight> {
        if self.distances[vertex as usize] != Weight::MAX {
            return Some(self.distances[vertex as usize]);
        }

        None
    }

    fn set_distance(&mut self, vertex: VertexId, distance: Weight) {
        self.distances[vertex as usize] = distance;
    }
}
