use crate::{
    data_structures::{VecVertexMap, VecVertexSet, VertexMap, VertexSet, reversed_path},
    graph::{EdgeLike, GraphLike},
    path::{Path, Query},
    pathfinder::ShortestPathFinder,
    types::{Distance, Vertex},
};

use std::{cmp::Reverse, collections::BinaryHeap};

/// Struct that contains everything a dijkstra search needs.
pub struct DijkstraPathfinder<'a, G, DistanceMap, PredecessorMap, ExpandedSet>
where
    G: GraphLike,
{
    graph: &'a G,
    queue: BinaryHeap<(Reverse<<G::Edge as EdgeLike>::Weight>, Vertex)>,
    distance: DistanceMap,
    predecessor: PredecessorMap,
    expanded: ExpandedSet,
}

impl<'a, G, D> DijkstraPathfinder<'a, G, VecVertexMap<D>, VecVertexMap<Vertex>, VecVertexSet>
where
    G: GraphLike<Edge: EdgeLike<Weight = D>>,
    D: Distance,
{
    pub fn new(graph: &'a G) -> Self {
        let len = graph.num_vertices();
        Self {
            graph,
            queue: BinaryHeap::new(),
            distance: VecVertexMap::new(len, D::max_value()),
            predecessor: VecVertexMap::new(len, Vertex::MAX),
            expanded: VecVertexSet::new(len),
        }
    }
}

impl<'a, G, D, DistanceMap, PredecessorMap, ExpandedSet>
    DijkstraPathfinder<'a, G, DistanceMap, PredecessorMap, ExpandedSet>
where
    G: GraphLike<Edge: EdgeLike<Weight = D>>,
    D: Distance,
    DistanceMap: VertexMap<D>,
    PredecessorMap: VertexMap<Vertex>,
    ExpandedSet: VertexSet,
{
    fn search(&mut self, query: &Query) -> Option<D> {
        self.queue.clear();
        self.distance.clear();
        self.predecessor.clear();
        self.expanded.clear();

        self.queue.push((Reverse(D::zero()), query.source));
        self.distance.set(query.source, D::zero());

        while let Some((Reverse(dist_tail), tail)) = self.queue.pop() {
            if self.expanded.contains_and_insert(tail) {
                continue;
            }

            if tail == query.target {
                return Some(dist_tail);
            }

            for edge in self.graph.outgoing_edges(tail) {
                let new_distance = dist_tail + edge.weight();
                let current_distance = self.distance.get(edge.head());
                if current_distance.is_some_and(|distance| new_distance >= distance) {
                    continue;
                }

                self.distance.set(edge.head(), new_distance);
                self.predecessor.set(edge.head(), tail);
                self.queue.push((Reverse(new_distance), edge.head()));
            }
        }

        None
    }
}

impl<'a, G, D, DistanceMap, PredecessorMap, ExpandedSet> ShortestPathFinder
    for DijkstraPathfinder<'a, G, DistanceMap, PredecessorMap, ExpandedSet>
where
    G: GraphLike<Edge: EdgeLike<Weight = D>>,
    D: Distance,
    DistanceMap: VertexMap<D>,
    PredecessorMap: VertexMap<Vertex>,
    ExpandedSet: VertexSet,
{
    type Distance = D;
    fn path(&mut self, query: &Query) -> Option<Path<Self::Distance>> {
        let distance = self.search(query)?;
        let mut vertices = reversed_path(&self.predecessor, query.target);
        vertices.reverse();
        Some(Path { vertices, distance })
    }

    fn distance(&mut self, query: &Query) -> Option<Self::Distance> {
        self.search(query)
    }
}
