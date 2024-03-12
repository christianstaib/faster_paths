use crate::{
    graphs::{
        fast_graph::FastGraph,
        path::{Path, PathFinding, ShortestPathRequest},
        types::{VertexId, Weight},
    },
    queue::{bucket_queue::BucketQueue, radix_queue::RadixQueue, State},
};

#[derive(Clone)]
pub struct FastDijkstra<'a> {
    graph: &'a FastGraph,
    max_edge_weight: Weight,
}

impl<'a> PathFinding for FastDijkstra<'a> {
    fn get_shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (weights, predecessors) = self.get_data(route_request.source(), route_request.target());

        let weight = *weights.get(route_request.target() as usize)?;
        if weight == u32::MAX {
            return None;
        }

        let mut vertices = vec![route_request.target()];
        let mut current = route_request.target();
        while let Some(&predecessor) = predecessors.get(current as usize) {
            if predecessor == u32::MAX {
                break;
            }
            current = predecessor;
            vertices.push(current);
        }
        vertices.reverse();
        Some(Path { weight, vertices })
    }

    fn get_shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.get_shortest_path(path_request)?;
        Some(data.weight)
    }
}

impl<'a> FastDijkstra<'a> {
    pub fn new(graph: &'a FastGraph) -> FastDijkstra {
        let max_edge_weight = graph.max_edge_weight().unwrap_or(0);
        FastDijkstra {
            graph,
            max_edge_weight,
        }
    }

    pub fn get_data(&self, source: VertexId, target: VertexId) -> (Vec<u32>, Vec<VertexId>) {
        let mut queue = RadixQueue::new(); //BucketQueue::new(self.max_edge_weight);
        let mut weights = vec![u32::MAX; self.graph.number_of_vertices() as usize];
        let mut predcessors = vec![u32::MAX; self.graph.number_of_vertices() as usize];
        let mut expanded = vec![false; self.graph.number_of_vertices() as usize];

        queue.push(State::new(0, source));
        weights[source as usize] = 0;

        while let Some(State { vertex, .. }) = queue.pop() {
            if vertex == target {
                break;
            }
            if expanded[vertex as usize] {
                continue;
            }
            expanded[vertex as usize] = true;

            for edge in self.graph.out_edges(vertex).iter() {
                let alternative_weight = weights[vertex as usize] + edge.weight;
                let current_weight = weights[edge.head as usize];
                if alternative_weight < current_weight {
                    queue.push(State::new(alternative_weight, edge.head));
                    weights[edge.head as usize] = alternative_weight;
                    predcessors[edge.head as usize] = vertex;
                }
            }
        }

        (weights, predcessors)
    }
}
