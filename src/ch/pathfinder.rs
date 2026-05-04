use crate::ch::contraction_hierarchy::ContractionHierarchy;
use crate::ch::edge::Edge;
use crate::flattened_nested::FlattenedNested;
use crate::path::{Path, PathQuery};
use crate::search_state::hash_search_state::HashSearchState;
use crate::search_state::search_state_access::SearchStateAccess;
use crate::types::{Distance, VertexId};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub enum Direction {
    UP,
    DOWN,
}

pub struct Pathfinder<'a> {
    contraction_hierarchy: &'a ContractionHierarchy,
    queue: BinaryHeap<(Reverse<Distance>, VertexId, Direction)>,
    up_state: HashSearchState,
    down_state: HashSearchState,
}

/// Indicates if `vertex` can be stalled.
///
/// Given the dir1 search state, performs a 1-hop search from `vertex` in dir2.
/// This provides a lower bound to the distance from the implicitly given start
/// vertex of dir1 to `vertex`. If `dir1_dist_vertex` violates this lower bound,
/// it cannot be optimal and `vertex` can be stalled in dir1.
fn stall(
    dir1_state: &HashSearchState,
    dir2_graph: &FlattenedNested<Edge>,
    vertex: VertexId,
    dir1_dist_vertex: Distance,
) -> bool {
    for edge in dir2_graph.nested(vertex.as_usize()) {
        if let Some(dir1_dist_meeting_vertex) = dir1_state.get_distance(edge.head()) {
            if dir1_dist_meeting_vertex + edge.weight() < dir1_dist_vertex {
                return true;
            }
        }
    }

    false
}

impl<'a> Pathfinder<'a> {
    pub fn new(
        contraction_hierarchy: &'a ContractionHierarchy,
        queue: BinaryHeap<(Reverse<Distance>, VertexId, Direction)>,
        up_state: HashSearchState,
        down_state: HashSearchState,
    ) -> Self {
        Self {
            contraction_hierarchy,
            queue,
            up_state,
            down_state,
        }
    }

    pub fn search(&mut self, query: &PathQuery) -> Option<(Distance, VertexId)> {
        // Set up the data structures for the search, just like in a normal bidirectional search.
        self.queue.clear();
        self.queue
            .push((Reverse(Distance::ZERO), query.source(), Direction::UP));
        self.queue
            .push((Reverse(Distance::ZERO), query.target(), Direction::DOWN));

        self.up_state.clear();
        self.up_state.set_distance(query.source(), Distance::ZERO);

        self.down_state.clear();
        self.down_state.set_distance(query.target(), Distance::ZERO);

        let mut best_meeting: Option<(Distance, VertexId)> = None;

        while let Some((Reverse(dir1_dist_tail), tail, dir1)) = self.queue.pop() {
            // Once all distances in the queue are larger than the meeting distance,
            // no shorter path can be found.
            if best_meeting.is_some_and(|(distance, _vertex)| dir1_dist_tail >= distance) {
                break;
            }

            // Set up the variables to use the same code for both directions.
            let dir1_is_upward = dir1 == Direction::UP;
            let (dir1_state, dir2_state, dir1_graph, dir2_graph) = if dir1_is_upward {
                (
                    &mut self.up_state,
                    &self.down_state,
                    self.contraction_hierarchy.up_graph(),
                    self.contraction_hierarchy.down_graph(),
                )
            } else {
                (
                    &mut self.down_state,
                    &self.up_state,
                    self.contraction_hierarchy.down_graph(),
                    self.contraction_hierarchy.up_graph(),
                )
            };

            // Skip the vertex if it has already been expanded.
            if dir1_state.is_expanded(tail) {
                continue;
            }
            dir1_state.set_expanded(tail);

            // Skip if dir1_dist_tail is not optimal, as this implies that every new_best_distance
            // would not be optimal.
            if stall(dir1_state, dir2_graph, tail, dir1_dist_tail) {
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
            for edge in dir1_graph.nested(tail.as_usize()) {
                let new_distance = dir1_dist_tail + edge.weight();
                let current_distance = dir1_state.get_distance(edge.head());
                if current_distance.is_none_or(|distance| new_distance < distance) {
                    dir1_state.set_distance(edge.head(), new_distance);
                    dir1_state.set_predecessor(edge.head(), tail);
                    self.queue.push((Reverse(new_distance), edge.head(), dir1));
                }
            }
        }

        best_meeting
    }

    pub fn path(&mut self, query: &PathQuery) -> Option<Path> {
        let (distance, vertex) = self.search(query)?;
        let mut up_path = self.up_state.get_path(vertex)?;
        let down_path = self.down_state.get_path(vertex)?;

        up_path.reverse();
        up_path.extend(down_path);

        Some(Path::new(up_path, distance))
    }
}
