use crate::ch::contraction_hierarchy::ContractionHierarchy;
use crate::ch::edge::Edge;
use crate::ch::shortcut::unpack_and_concat_shortcut_paths;
use crate::flattened_nested::FlattenedNested;
use crate::path::{Path, PathQuery};
use crate::pathfinder::ShortestPathFinder;
use crate::search_state::hash_search_state::HashSearchState;
use crate::search_state::search_state_access::SearchStateAccess;
use crate::types::{Distance, VertexId};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
enum Direction {
    UP,
    DOWN,
}

pub struct ContractionHierarchyPathfinder<'a> {
    contraction_hierarchy: &'a ContractionHierarchy,
    queue: BinaryHeap<(Reverse<Distance>, VertexId, Direction)>,
    up_state: HashSearchState,
    down_state: HashSearchState,
}

impl<'a> ShortestPathFinder for ContractionHierarchyPathfinder<'a> {
    fn path(&mut self, query: &PathQuery) -> Option<Path> {
        let (distance, meeting_vertex) = self.search(query)?;

        let up_reversed_shortcut_path = self.up_state.get_reversed_path(meeting_vertex)?;
        let down_reversed_shortcut_path = self.down_state.get_reversed_path(meeting_vertex)?;
        let path = unpack_and_concat_shortcut_paths(
            self.contraction_hierarchy,
            &up_reversed_shortcut_path,
            &down_reversed_shortcut_path,
        )?;

        Some(Path::new(path, distance))
    }

    fn distance(&mut self, query: &PathQuery) -> Option<Distance> {
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

impl<'a> ContractionHierarchyPathfinder<'a> {
    pub fn new(contraction_hierarchy: &'a ContractionHierarchy) -> Self {
        Self {
            contraction_hierarchy,
            queue: BinaryHeap::new(),
            up_state: HashSearchState::new(),
            down_state: HashSearchState::new(),
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
            let (dir1_state, dir2_state, dir1_graph, dir2_graph) = match dir1 {
                Direction::UP => (
                    &mut self.up_state,
                    &self.down_state,
                    self.contraction_hierarchy.up_graph(),
                    self.contraction_hierarchy.down_graph(),
                ),
                Direction::DOWN => (
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
            for edge in dir1_graph.nested(tail.as_usize()) {
                let new_distance = dir1_dist_tail + edge.weight();
                let current_distance = dir1_state.get_distance(edge.head());
                if !current_distance.is_none_or(|distance| new_distance < distance) {
                    continue;
                }

                dir1_state.set_distance(edge.head(), new_distance);
                dir1_state.set_predecessor(edge.head(), tail);
                self.queue.push((Reverse(new_distance), edge.head(), dir1));
            }
        }

        best_meeting
    }
}
