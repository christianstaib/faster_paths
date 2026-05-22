use std::{cmp::Reverse, collections::BinaryHeap};

use num_traits::Zero;

use crate::{
    data_structures::SearchStateAccess,
    graph::{EdgeLike, GraphLike},
    path::{Path, Query},
    pathfinder::ShortestPathFinder,
    types::Vertex,
};

pub struct DijkstraPathfinder<'a, G, S>
where
    G: GraphLike,
    S: SearchStateAccess<<G::Edge as EdgeLike>::Weight>,
{
    graph: &'a G,
    queue: BinaryHeap<(Reverse<<G::Edge as EdgeLike>::Weight>, Vertex)>,
    search_state: S,
}

impl<'a, G, S> DijkstraPathfinder<'a, G, S>
where
    G: GraphLike,
    S: SearchStateAccess<<G::Edge as EdgeLike>::Weight>,
{
    pub fn new(graph: &'a G) -> Self {
        Self {
            graph,
            queue: BinaryHeap::new(),
            search_state: S::new(graph),
        }
    }

    fn search(&mut self, query: &Query) -> Option<<G::Edge as EdgeLike>::Weight> {
        self.queue.clear();
        self.queue
            .push((Reverse(<G::Edge as EdgeLike>::Weight::zero()), query.source));

        self.search_state.clear();
        self.search_state
            .set_distance(query.source, <G::Edge as EdgeLike>::Weight::zero());

        while let Some((Reverse(dist_tail), tail)) = self.queue.pop() {
            if self.search_state.test_and_set_expanded(tail) {
                continue;
            }

            if tail == query.target {
                return Some(dist_tail);
            }

            for edge in self.graph.outgoing_edges(tail) {
                let new_distance = dist_tail + edge.weight();
                let current_distance = self.search_state.get_distance(edge.head());
                if current_distance.is_some_and(|distance| new_distance >= distance) {
                    continue;
                }

                self.search_state.set_distance(edge.head(), new_distance);
                self.search_state.set_predecessor(edge.head(), tail);
                self.queue.push((Reverse(new_distance), edge.head()));
            }
        }

        None
    }
}

impl<'a, G, S> ShortestPathFinder for DijkstraPathfinder<'a, G, S>
where
    G: GraphLike,
    S: SearchStateAccess<<G::Edge as EdgeLike>::Weight>,
{
    type Distance = <G::Edge as EdgeLike>::Weight;

    fn path(&mut self, query: &Query) -> Option<Path<Self::Distance>> {
        let distance = self.search(query)?;
        let mut vertices = self.search_state.get_reversed_path(query.target)?;
        vertices.reverse();

        Some(Path { vertices, distance })
    }

    fn distance(&mut self, query: &Query) -> Option<Self::Distance> {
        self.search(query)
    }
}
