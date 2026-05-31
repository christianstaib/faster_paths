use crate::contraction_hierarchy::contraction_hierarchy::ContractionHierarchy;
use crate::contraction_hierarchy::shortcut::unpack_and_concat_shortcut_paths;
use crate::data_structures::{HashSearchState, SearchStateAccess};
use crate::graph::{EdgeLike, GraphLike};
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

#[derive(Eq, PartialEq)]
struct Entry<D>(Reverse<D>, Vertex, Direction);

impl<D: Ord> Ord for Entry<D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<D: Ord> PartialOrd for Entry<D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ContractionHierarchyPathfinder<'a, D: Distance> {
    contraction_hierarchy: &'a ContractionHierarchy<D>,
    queue: BinaryHeap<Entry<D>>,
    up_state: HashSearchState<D>,
    down_state: HashSearchState<D>,
}

impl<'a, D: Distance> ShortestPathFinder for ContractionHierarchyPathfinder<'a, D> {
    type Distance = D;

    fn path(&mut self, query: &Query) -> Option<Path<D>> {
        let (distance, meeting_vertex) = self.search(query)?;

        let up_reversed_shortcut_path = self.up_state.get_reversed_path(meeting_vertex)?;
        let down_reversed_shortcut_path = self.down_state.get_reversed_path(meeting_vertex)?;

        let vertices = unpack_and_concat_shortcut_paths(
            self.contraction_hierarchy,
            &up_reversed_shortcut_path,
            &down_reversed_shortcut_path,
            self.contraction_hierarchy.num_edges() * 2,
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
    dir1_state: &impl SearchStateAccess<D>,
    dir2_graph: &impl GraphLike<Edge: EdgeLike<Weight = D>>,
    vertex: Vertex,
    dir1_dist_vertex: D,
) -> bool {
    for edge in dir2_graph.outgoing_edges(vertex) {
        if let Some(dir1_dist_meeting_vertex) = dir1_state.get_distance(edge.head())
            && dir1_dist_meeting_vertex + edge.weight() < dir1_dist_vertex
        {
            return true;
        }
    }

    false
}

impl<'a, D: Distance> ContractionHierarchyPathfinder<'a, D> {
    pub fn new(contraction_hierarchy: &'a ContractionHierarchy<D>) -> Self {
        Self {
            contraction_hierarchy,
            queue: BinaryHeap::new(),
            up_state: HashSearchState::new(),
            down_state: HashSearchState::new(),
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
            .push(Entry(Reverse(D::zero()), query.source, Direction::Up));
        self.queue
            .push(Entry(Reverse(D::zero()), query.target, Direction::Down));

        self.up_state.clear();
        self.up_state.set_distance(query.source, D::zero());

        self.down_state.clear();
        self.down_state.set_distance(query.target, D::zero());

        let mut best_meeting: Option<(D, Vertex)> = None;

        while let Some(Entry(Reverse(dir1_dist_tail), tail, dir1)) = self.queue.pop() {
            // Once all distances in the queue are larger than the meeting distance,
            // no shorter path can be found.
            if best_meeting.is_some_and(|(distance, _vertex)| dir1_dist_tail >= distance) {
                break;
            }

            // Set up the variables to use the same code for both directions.
            let (dir1_state, dir2_state, dir1_graph, dir2_graph) = match dir1 {
                Direction::Up => (
                    &mut self.up_state,
                    &self.down_state,
                    self.contraction_hierarchy.up_graph(),
                    self.contraction_hierarchy.down_graph(),
                ),
                Direction::Down => (
                    &mut self.down_state,
                    &self.up_state,
                    self.contraction_hierarchy.down_graph(),
                    self.contraction_hierarchy.up_graph(),
                ),
            };

            // Skip the vertex if it has already been expanded.
            // Skip if dir1_dist_tail is not optimal, as this implies that every new_best_distance
            // would not be optimal.
            if dir1_state.test_and_set_expanded(tail)
                || stall(dir1_state, dir2_graph, tail, dir1_dist_tail)
            {
                continue;
            }

            // Check whether a better meeting distance has been found.
            if let Some(dir2_dist_tail) = dir2_state.get_distance(tail) {
                let new_best_distance = dir1_dist_tail + dir2_dist_tail;
                if best_meeting.is_none_or(|(distance, _vertex)| new_best_distance < distance) {
                    best_meeting = Some((new_best_distance, tail));
                }
            }

            // Perform normal edge relaxation.
            for edge in dir1_graph.outgoing_edges(tail) {
                let new_distance = dir1_dist_tail + edge.weight;
                let current_distance = dir1_state.get_distance(edge.head);
                if current_distance.is_some_and(|current_distance| new_distance >= current_distance)
                {
                    continue;
                }

                dir1_state.set_distance(edge.head, new_distance);
                dir1_state.set_predecessor(edge.head, tail);
                self.queue
                    .push(Entry(Reverse(new_distance), edge.head, dir1));
            }
        }

        best_meeting
    }
}
