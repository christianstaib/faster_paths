use crate::{
    graph::GraphLike,
    types::{Distance, Vertex},
};

pub trait SearchStateAccess<D: Distance> {
    fn new<G: GraphLike>(graph: &G) -> Self;

    fn get_distance(&self, vertex: Vertex) -> Option<D>;
    fn set_distance(&mut self, vertex: Vertex, distance: D);

    fn get_predecessor(&self, vertex: Vertex) -> Option<Vertex>;
    fn set_predecessor(&mut self, vertex: Vertex, predecessor: Vertex);

    fn is_expanded(&self, vertex: Vertex) -> bool;
    fn set_expanded(&mut self, vertex: Vertex);

    fn test_and_set_expanded(&mut self, vertex: Vertex) -> bool {
        let is_expanded = self.is_expanded(vertex);
        self.set_expanded(vertex);
        is_expanded
    }

    fn clear(&mut self);

    /// If target is reachable, returns the reversed path, e.g. [target, ..., source], otherwise None.
    fn get_reversed_path(&self, target: Vertex) -> Option<Vec<Vertex>> {
        self.get_distance(target)?;

        let mut path = Vec::new();
        let mut current = target;

        path.push(current);

        while let Some(predecessor) = self.get_predecessor(current) {
            current = predecessor;
            path.push(current);
        }

        Some(path)
    }
}
