use std::{cmp::Reverse, collections::BinaryHeap};

use keyed_priority_queue::KeyedPriorityQueue;

use crate::graphs::{Distance, Vertex};

/// A trait for a priority queue that manages vertices and their distances.
/// This trait is useful for graph algorithms that need to repeatedly retrieve
/// the vertex with the smallest distance (such as Dijkstra's algorithm).
///
/// The implementing structs might or might not use a decrease key operation.
pub trait VertexDistanceQueue {
    /// Clears all stored data, preparing for a new search.
    fn clear(&mut self);

    /// Inserts a vertex with its associated distance into the priority queue.
    fn insert(&mut self, vertex: Vertex, distance: Distance);

    /// Removes and returns the vertex with the smallest distance from the
    /// priority queue or none if the queue is empty.
    fn pop(&mut self) -> Option<(Vertex, Distance)>;

    fn is_empty(&self) -> bool;

    fn peek(&mut self) -> Option<(Vertex, Distance)>;
}

/// A priority queue implementation using thre rust collections Binary Heap.
pub struct VertexDistanceQueueBinaryHeap {
    heap: BinaryHeap<Reverse<(Distance, Vertex)>>,
}

impl VertexDistanceQueueBinaryHeap {
    pub fn new() -> Self {
        VertexDistanceQueueBinaryHeap {
            heap: BinaryHeap::new(),
        }
    }
}

impl VertexDistanceQueue for VertexDistanceQueueBinaryHeap {
    fn clear(&mut self) {
        self.heap.clear();
    }

    fn insert(&mut self, vertex: Vertex, distance: Distance) {
        self.heap.push(Reverse((distance, vertex)));
    }

    fn pop(&mut self) -> Option<(Vertex, Distance)> {
        let Reverse((distance, vertex)) = self.heap.pop()?;

        Some((vertex, distance))
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    fn peek(&mut self) -> Option<(Vertex, Distance)> {
        let &Reverse((distance, vertex)) = self.heap.peek()?;

        Some((vertex, distance))
    }
}

pub struct VertexDistanceKeyedPriorityQueue {
    queue: KeyedPriorityQueue<Vertex, Reverse<Distance>>,
}

impl VertexDistanceKeyedPriorityQueue {
    pub fn new() -> Self {
        VertexDistanceKeyedPriorityQueue {
            queue: KeyedPriorityQueue::new(),
        }
    }
}

impl VertexDistanceQueue for VertexDistanceKeyedPriorityQueue {
    fn clear(&mut self) {
        self.queue.clear();
    }

    fn insert(&mut self, vertex: Vertex, distance: Distance) {
        self.queue.push(vertex, Reverse(distance));
    }

    fn pop(&mut self) -> Option<(Vertex, Distance)> {
        self.queue
            .pop()
            .map(|(vertex, Reverse(distance))| (vertex, distance))
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn peek(&mut self) -> Option<(Vertex, Distance)> {
        self.queue
            .peek()
            .map(|(&vertex, &Reverse(distance))| (vertex, distance))
    }
}
