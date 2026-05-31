use crate::contraction_hierarchy::contraction_hierarchy::ContractionHierarchy;
use crate::contraction_hierarchy::edge::ContractionEdge;
use crate::contraction_hierarchy::shortcut::unpack_and_concat_shortcut_paths;
use crate::data_structures::{HashVertexMap, HashVertexSet, VertexMap, VertexSet, reversed_path};
use crate::graph::{CsrGraph, EdgeLike, GraphLike};
use crate::path::{Path, Query};
use crate::pathfinder::ShortestPathFinder;
use crate::types::{Distance, Vertex};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
enum Direction {
    Up,
    Down,
}

/// Search state for one direction of a Contraction Hierarchy query. Just a wrapper so the search
/// becomes more readable.
struct ChSearchState<'a, D: Distance> {
    graph: &'a CsrGraph<ContractionEdge<D>>,
    distance: HashVertexMap<D>,
    predecessor: HashVertexMap<Vertex>,
    expanded: HashVertexSet,
}

/// Path finder for shortest-path queries on a precomputed Contraction Hierarchy.
/// Reuses its internal data structures across queries.
pub struct ContractionHierarchyPathfinder<'a, D: Distance> {
    contraction_hierarchy: &'a ContractionHierarchy<D>,
    queue: BinaryHeap<(Reverse<D>, Vertex, Direction)>,
    up_state: ChSearchState<'a, D>,
    down_state: ChSearchState<'a, D>,
}

impl<'a, D: Distance> ChSearchState<'a, D> {
    fn new(graph: &'a CsrGraph<ContractionEdge<D>>, len: usize) -> Self {
        Self {
            graph,
            distance: HashVertexMap::new(len, D::max_value()),
            predecessor: HashVertexMap::new(len, Vertex::MAX),
            expanded: HashVertexSet::new(len),
        }
    }

    fn clear(&mut self) {
        self.distance.clear();
        self.predecessor.clear();
        self.expanded.clear();
    }
}

impl<'a, D: Distance> ShortestPathFinder for ContractionHierarchyPathfinder<'a, D> {
    type Distance = D;

    fn path(&mut self, query: &Query) -> Option<Path<D>> {
        let (distance, meeting_vertex) = self.search(query)?;

        let up_predecessor = &self.up_state.predecessor;
        let up_reversed_shortcut_path = reversed_path(up_predecessor, meeting_vertex);

        let down_predecessor = &self.down_state.predecessor;
        let down_reversed_shortcut_path = reversed_path(down_predecessor, meeting_vertex);

        let unpack_limit = self.contraction_hierarchy.num_edges() * 2;
        let vertices = unpack_and_concat_shortcut_paths(
            self.contraction_hierarchy,
            &up_reversed_shortcut_path,
            &down_reversed_shortcut_path,
            unpack_limit,
        )?;

        Some(Path { vertices, distance })
    }

    fn distance(&mut self, query: &Query) -> Option<D> {
        let (distance, _meeting_vertex) = self.search(query)?;

        Some(distance)
    }
}

/// Indicates if `vertex` can be stalled.
///
/// Given the dir1 search state, performs a 1-hop search from `vertex` in dir2.
/// This provides a lower bound to the distance from the implicitly given start
/// vertex of dir1 to `vertex`. If `dir1_dist_vertex` violates this lower bound,
/// it cannot be optimal and `vertex` can be stalled in dir1.
fn stall<D: Distance>(
    dir1_distance: &impl VertexMap<D>,
    dir2_graph: &impl GraphLike<Edge: EdgeLike<Weight = D>>,
    vertex: Vertex,
    dir1_dist_vertex: D,
) -> bool {
    for edge in dir2_graph.outgoing_edges(vertex) {
        if let Some(dir1_dist_meeting_vertex) = dir1_distance.get(edge.head())
            && dir1_dist_meeting_vertex + edge.weight() < dir1_dist_vertex
        {
            return true;
        }
    }

    false
}

impl<'a, D: Distance> ContractionHierarchyPathfinder<'a, D> {
    pub fn new(contraction_hierarchy: &'a ContractionHierarchy<D>) -> Self {
        let len = contraction_hierarchy.num_vertices();

        Self {
            contraction_hierarchy,
            queue: BinaryHeap::new(),
            up_state: ChSearchState::new(contraction_hierarchy.up_graph(), len),
            down_state: ChSearchState::new(contraction_hierarchy.down_graph(), len),
        }
    }

    /// Runs a bidirectional Contraction Hierarchy query.
    ///
    /// Vertices reached from both directions become meeting candidates, and the
    /// search stops once the queued distances cannot improve the best candidate.
    ///
    /// Returns the shortest distance together with its meeting vertex, or `None`
    /// if no path exists. On success, `up_state` and `down_state` retain the
    /// predecessor chains needed to reconstruct and unpack the shortcut path
    /// through the returned meeting vertex.
    fn search(&mut self, query: &Query) -> Option<(D, Vertex)> {
        // Set up the data structures for the search, just like in a normal bidirectional search.
        self.queue.clear();
        self.queue
            .push((Reverse(D::zero()), query.source, Direction::Up));
        self.queue
            .push((Reverse(D::zero()), query.target, Direction::Down));

        self.up_state.clear();
        self.up_state.distance.set(query.source, D::zero());

        self.down_state.clear();
        self.down_state.distance.set(query.target, D::zero());

        let mut best_meeting: Option<(D, Vertex)> = None;

        while let Some((Reverse(dir1_dist_tail), tail, dir1)) = self.queue.pop() {
            // Once all distances in the queue are larger than the meeting distance,
            // no shorter path can be found.
            if best_meeting.is_some_and(|(distance, _vertex)| dir1_dist_tail >= distance) {
                break;
            }

            // Set up the variables to use the same code for both directions.
            let (dir1_state, dir2_state) = match dir1 {
                Direction::Up => (&mut self.up_state, &self.down_state),
                Direction::Down => (&mut self.down_state, &self.up_state),
            };

            // Skip the vertex if it has already been expanded.
            // Skip if dir1_dist_tail is not optimal, as this implies that every new_best_distance
            // would not be optimal.
            if dir1_state.expanded.contains_and_insert(tail)
                || stall(&dir1_state.distance, dir2_state.graph, tail, dir1_dist_tail)
            {
                continue;
            }

            // Check whether a better meeting distance has been found.
            if let Some(dir2_dist_tail) = dir2_state.distance.get(tail) {
                let new_best_distance = dir1_dist_tail + dir2_dist_tail;
                if best_meeting.is_none_or(|(distance, _vertex)| new_best_distance < distance) {
                    best_meeting = Some((new_best_distance, tail));
                }
            }

            // Perform normal edge relaxation.
            for edge in dir1_state.graph.outgoing_edges(tail) {
                let new_distance = dir1_dist_tail + edge.weight;
                let current_distance = dir1_state.distance.get(edge.head);
                if current_distance.is_some_and(|current_distance| new_distance >= current_distance)
                {
                    continue;
                }

                dir1_state.distance.set(edge.head, new_distance);
                dir1_state.predecessor.set(edge.head, tail);
                self.queue.push((Reverse(new_distance), edge.head, dir1));
            }
        }

        best_meeting
    }
}
