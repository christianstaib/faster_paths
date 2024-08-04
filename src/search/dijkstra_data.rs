use crate::graphs::{Graph, VertexId, Weight};

pub struct Path {
    pub vertices: Vec<VertexId>,
    pub weight: Weight,
}

pub trait DijkstraData {
    fn clear(&mut self);

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId>;

    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId);

    fn get_distance(&self, vertex: VertexId) -> Option<VertexId>;

    fn set_distance(&mut self, vertex: VertexId, distance: Weight);

    fn get_path(&self, target: VertexId) -> Option<Path> {
        let mut path = Path {
            vertices: Vec::new(),
            weight: self.get_distance(target)?,
        };

        let mut predecessor = target;
        path.vertices.push(predecessor);
        while let Some(new_predecessor) = self.get_predecessor(predecessor) {
            predecessor = new_predecessor;
            path.vertices.push(predecessor);
        }

        path.vertices.reverse();

        Some(path)
    }
}

pub struct DijkstraDataVec {
    // only one vector for better cache locality
    predecessors_and_distances: Vec<(VertexId, Weight)>,
}

impl DijkstraDataVec {
    pub fn new(graph: &dyn Graph) -> Self {
        DijkstraDataVec {
            predecessors_and_distances: vec![
                (VertexId::MAX, Weight::MAX);
                graph.number_of_vertices() as usize
            ],
        }
    }
}

impl DijkstraData for DijkstraDataVec {
    fn clear(&mut self) {
        self.predecessors_and_distances
            .fill((VertexId::MAX, Weight::MAX));
    }

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId> {
        let predecessor = self.predecessors_and_distances[vertex as usize].0;
        if predecessor != VertexId::MAX {
            return Some(predecessor);
        }

        None
    }

    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId) {
        self.predecessors_and_distances[vertex as usize].0 = predecessor;
    }

    fn get_distance(&self, vertex: VertexId) -> Option<Weight> {
        let distance = self.predecessors_and_distances[vertex as usize].1;
        if distance != Weight::MAX {
            return Some(distance);
        }

        None
    }

    fn set_distance(&mut self, vertex: VertexId, distance: Weight) {
        self.predecessors_and_distances[vertex as usize].1 = distance;
    }
}
