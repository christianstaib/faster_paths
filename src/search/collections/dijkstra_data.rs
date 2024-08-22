use std::collections::HashMap;

use crate::graphs::{Distance, Graph, Vertex};

#[derive(Debug)]
pub struct Path {
    pub vertices: Vec<Vertex>,
    pub distance: Distance,
}

/// Trait for handling data access in Dijkstra's algorithm.
pub trait DijkstraData {
    /// Clears all stored data, preparing for a new search.
    fn clear(&mut self);

    /// Retrieves the predecessor of a given vertex, if any.
    fn get_predecessor(&self, vertex: Vertex) -> Option<Vertex>;

    /// Sets the predecessor for a given vertex.
    fn set_predecessor(&mut self, vertex: Vertex, predecessor: Vertex);

    /// Retrieves the distance to a given vertex, if any.
    fn get_distance(&self, vertex: Vertex) -> Option<Distance>;

    /// Sets the distance to a given vertex.
    fn set_distance(&mut self, vertex: Vertex, distance: Distance);

    /// Constructs the path to a target vertex, if reachable.
    ///
    /// This function traces back from the target vertex using
    /// predecessor data to build the full path. Returns `None`
    /// if the target vertex is unreachable.
    fn get_path(&self, target: Vertex) -> Option<Path> {
        // Retrieve the distance to the target vertex.
        let distance = self.get_distance(target)?;
        // Initialize the path with the target vertex.
        let mut vertices = vec![target];

        // Start tracing back from the target vertex.
        let mut predecessor = target;
        while let Some(new_predecessor) = self.get_predecessor(predecessor) {
            predecessor = new_predecessor;
            vertices.push(predecessor);
        }

        // Reverse the path to start from the source vertex.
        vertices.reverse();

        // Create a Path object with the traced vertices and the distance.
        let path = Path { vertices, distance };

        Some(path)
    }
}

/// Struct to store predecessors and distances in a single vector.
pub struct DijkstraDataVec {
    // A vector storing tuples of (predecessor, distance) for each vertex.
    predecessors_and_distances: Vec<(Option<Vertex>, Option<Distance>)>,
}

impl DijkstraDataVec {
    /// Constructs a new `DijkstraDataVec` for a given graph.
    pub fn new(graph: &dyn Graph) -> Self {
        DijkstraDataVec {
            // Initialize the vector with (None, None) tuples for each vertex in the graph.
            predecessors_and_distances: vec![(None, None); graph.number_of_vertices() as usize],
        }
    }
}

impl DijkstraData for DijkstraDataVec {
    fn clear(&mut self) {
        self.predecessors_and_distances.fill((None, None));
    }

    fn get_predecessor(&self, vertex: Vertex) -> Option<Vertex> {
        self.predecessors_and_distances[vertex as usize].0
    }

    fn set_predecessor(&mut self, vertex: Vertex, predecessor: Vertex) {
        self.predecessors_and_distances[vertex as usize].0 = Some(predecessor);
    }

    fn get_distance(&self, vertex: Vertex) -> Option<Distance> {
        self.predecessors_and_distances[vertex as usize].1
    }

    fn set_distance(&mut self, vertex: Vertex, distance: Distance) {
        self.predecessors_and_distances[vertex as usize].1 = Some(distance);
    }
}

pub struct DijkstraDataHashMap {
    predecessors: HashMap<Vertex, Vertex>,
    distances: HashMap<Vertex, Distance>,
}

impl DijkstraDataHashMap {
    pub fn new() -> Self {
        DijkstraDataHashMap {
            predecessors: HashMap::new(),
            distances: HashMap::new(),
        }
    }
}

impl DijkstraData for DijkstraDataHashMap {
    fn clear(&mut self) {
        self.predecessors.clear();
        self.distances.clear();
    }

    fn get_predecessor(&self, vertex: Vertex) -> Option<Vertex> {
        self.predecessors.get(&vertex).cloned()
    }

    fn set_predecessor(&mut self, vertex: Vertex, predecessor: Vertex) {
        self.predecessors.insert(vertex, predecessor);
    }

    fn get_distance(&self, vertex: Vertex) -> Option<Distance> {
        self.distances.get(&vertex).cloned()
    }

    fn set_distance(&mut self, vertex: Vertex, distance: Distance) {
        self.distances.insert(vertex, distance);
    }
}
