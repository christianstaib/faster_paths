use std::{cmp::Reverse, collections::BinaryHeap};

use num_traits::Zero;

use crate::{
    graph::{EdgeLike, GraphLike},
    path::{Path, PathQuery},
    pathfinder::ShortestPathFinder,
    search_state::{HashSearchState, SearchStateAccess},
    types::VertexId,
};

pub struct DijkstraPathfinder<'a, G>
where
    G: GraphLike,
{
    graph: &'a G,
    queue: BinaryHeap<(Reverse<<G::Edge as EdgeLike>::Distance>, VertexId)>,
    up_state: HashSearchState<<G::Edge as EdgeLike>::Distance>,
}

impl<'a, G> DijkstraPathfinder<'a, G>
where
    G: GraphLike,
{
    pub fn new(graph: &'a G) -> Self {
        Self {
            graph,
            queue: BinaryHeap::new(),
            up_state: HashSearchState::new(),
        }
    }

    fn search(&mut self, query: &PathQuery) -> Option<<G::Edge as EdgeLike>::Distance> {
        self.queue.clear();
        self.queue.push((
            Reverse(<G::Edge as EdgeLike>::Distance::zero()),
            query.source,
        ));

        self.up_state.clear();
        self.up_state
            .set_distance(query.source, <G::Edge as EdgeLike>::Distance::zero());

        while let Some((Reverse(dist_tail), tail)) = self.queue.pop() {
            if self.up_state.test_and_set_expanded(tail) {
                continue;
            }

            if tail == query.target {
                return Some(dist_tail);
            }

            for edge in self.graph.out_edges(tail) {
                let new_distance = dist_tail + edge.weight();
                let current_distance = self.up_state.get_distance(edge.head());
                if current_distance.is_some_and(|distance| new_distance >= distance) {
                    continue;
                }

                self.up_state.set_distance(edge.head(), new_distance);
                self.up_state.set_predecessor(edge.head(), tail);
                self.queue.push((Reverse(new_distance), edge.head()));
            }
        }

        None
    }
}

impl<'a, G> ShortestPathFinder for DijkstraPathfinder<'a, G>
where
    G: GraphLike,
{
    type Distance = <G::Edge as EdgeLike>::Distance;

    fn path(&mut self, query: &PathQuery) -> Option<Path<Self::Distance>> {
        let distance = self.search(query)?;
        let mut vertices = self.up_state.get_reversed_path(query.target)?;
        vertices.reverse();

        Some(Path { vertices, distance })
    }

    fn distance(&mut self, query: &PathQuery) -> Option<Self::Distance> {
        self.search(query)
    }
}
