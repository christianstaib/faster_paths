use crate::types::{Distance, VertexId};

pub trait SearchStateAccess {
    fn get_distance(&self, vertex: VertexId) -> Option<Distance>;
    fn set_distance(&mut self, vertex: VertexId, distance: Distance);

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId>;
    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId);

    fn is_expanded(&self, vertex: VertexId) -> bool;
    fn set_expanded(&mut self, vertex: VertexId);

    fn clear(&mut self);

    fn get_path(&self, target: VertexId) -> Option<Vec<VertexId>> {
        self.get_distance(target)?;

        let mut path = Vec::new();
        let mut current = target;

        path.push(current);

        while let Some(predecessor) = self.get_predecessor(current) {
            current = predecessor;
            path.push(current);
        }

        path.reverse();
        Some(path)
    }
}
