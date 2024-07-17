use std::{collections::HashSet, usize};

use crate::{
    dijkstra_data::{dijkstra_data_vec::DijkstraDataVec, DijkstraData},
    graphs::{
        edge::{Edge, WeightedEdge},
        path::{Path, PathFinding, ShortestPathRequest},
        Graph, VertexId, Weight,
    },
    queue::DijkstraQueueElement,
};

pub struct Dijkstra {
    pub graph: Box<dyn Graph>,
}

impl PathFinding for Dijkstra {
    fn shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let data = get_data(&*self.graph, route_request.source(), route_request.target());
        data.get_path(route_request.target())
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let data = self.shortest_path(path_request)?;
        Some(data.weight)
    }

    fn number_of_vertices(&self) -> u32 {
        self.graph.number_of_vertices()
    }
}

pub fn shortest_path(graph: &dyn Graph, route_request: &ShortestPathRequest) -> Option<Path> {
    let data = get_data(graph, route_request.source(), route_request.target());
    data.get_path(route_request.target())
}

pub fn shortest_path_weight(
    graph: &dyn Graph,
    path_request: &ShortestPathRequest,
) -> Option<Weight> {
    shortest_path(graph, path_request).map(|path| path.weight)
}

pub fn get_data(graph: &dyn Graph, source: VertexId, target: VertexId) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        if vertex == target {
            return data;
        }
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    data
}

pub fn single_source(graph: &dyn Graph, source: VertexId) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    data
}

pub fn single_source_with_order(
    graph: &dyn Graph,
    source: VertexId,
    order: &[u32],
) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        if order[vertex as usize] <= order[source as usize] {
            continue;
        }
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    data
}

pub fn single_source_dijkstra_rank(
    graph: &dyn Graph,
    source: VertexId,
) -> (Vec<Option<u32>>, DijkstraDataVec) {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);
    let mut dijkstra_rank = vec![None; graph.number_of_vertices() as usize];

    let mut current_dijkstra_rank = 0;
    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        current_dijkstra_rank += 1;
        dijkstra_rank[vertex as usize] = Some(current_dijkstra_rank);
        graph
            .out_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.head(), edge.weight()));
    }

    (dijkstra_rank, data)
}

pub fn single_target(graph: &dyn Graph, target: VertexId) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, target);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        graph
            .in_edges(vertex)
            .for_each(|edge| data.update(vertex, edge.tail(), edge.weight()));
    }

    data
}

pub fn generate_upward_ch_edges(
    graph: &dyn Graph,
    source: VertexId,
    vertex_to_level_map: &[u32],
) -> (Vec<(Edge, VertexId)>, Vec<WeightedEdge>) {
    let mut alive = HashSet::new();
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, source);

    let mut shortcuts = Vec::new();
    let mut edges = Vec::new();

    alive.insert(source);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        if alive.contains(&vertex)
            && vertex_to_level_map[vertex as usize] > vertex_to_level_map[source as usize]
        {
            alive.remove(&vertex);

            let edge = WeightedEdge::new(
                source,
                vertex,
                data.vertices[vertex as usize].weight.unwrap(),
            )
            .unwrap();
            edges.push(edge.clone());

            let mut new_vertex = vertex;
            let mut predecessor = data.vertices[vertex as usize].predecessor.unwrap();
            if predecessor != source {
                loop {
                    let new_predecessor = data.vertices[predecessor as usize]
                        .predecessor
                        .unwrap_or(source);

                    let edge = Edge::new(source, new_vertex).unwrap();
                    shortcuts.push((edge, predecessor));
                    new_vertex = predecessor;
                    predecessor = new_predecessor;
                    if new_predecessor == source {
                        break;
                    }
                }
            }
        }

        if alive.is_empty() {
            break;
        }

        graph.out_edges(vertex).for_each(|edge| {
            let alt_weight = data.vertices[vertex as usize].weight.unwrap() + edge.weight();
            let cur_weight = data.vertices[edge.head() as usize]
                .weight
                .unwrap_or(u32::MAX);
            if alt_weight < cur_weight {
                if alive.contains(&vertex) {
                    alive.insert(edge.head());
                } else {
                    alive.remove(&edge.head());
                }

                data.vertices[edge.head() as usize].predecessor = Some(vertex);
                data.vertices[edge.head() as usize].weight = Some(alt_weight);
                data.queue
                    .push(DijkstraQueueElement::new(alt_weight, edge.head()));
            }
        });

        alive.remove(&vertex);
    }

    (shortcuts, edges)
}

pub fn generate_downward_ch_edges(
    graph: &dyn Graph,
    target: VertexId,
    vertex_to_level_map: &[u32],
) -> (Vec<(Edge, VertexId)>, Vec<WeightedEdge>) {
    let mut alive = HashSet::new();
    let mut data = DijkstraDataVec::new(graph.number_of_vertices() as usize, target);

    let mut shortcuts = Vec::new();
    let mut edges = Vec::new();

    alive.insert(target);

    while let Some(DijkstraQueueElement { vertex, .. }) = data.pop() {
        if alive.contains(&vertex)
            && vertex_to_level_map[vertex as usize] > vertex_to_level_map[target as usize]
        {
            alive.remove(&vertex);

            let edge = WeightedEdge::new(
                target,
                vertex,
                data.vertices[vertex as usize].weight.unwrap(),
            )
            .unwrap();
            edges.push(edge.clone());

            let mut new_vertex = vertex;
            let mut predecessor = data.vertices[vertex as usize].predecessor.unwrap();
            if predecessor != target {
                loop {
                    let new_predecessor = data.vertices[predecessor as usize]
                        .predecessor
                        .unwrap_or(target);

                    let edge = Edge::new(target, new_vertex).unwrap();
                    shortcuts.push((edge, predecessor));
                    new_vertex = predecessor;
                    predecessor = new_predecessor;
                    if new_predecessor == target {
                        break;
                    }
                }
            }
        }

        if alive.is_empty() {
            break;
        }

        graph.in_edges(vertex).for_each(|edge| {
            let alt_weight = data.vertices[vertex as usize].weight.unwrap() + edge.weight();
            let cur_weight = data.vertices[edge.tail() as usize]
                .weight
                .unwrap_or(u32::MAX);
            if alt_weight < cur_weight {
                if alive.contains(&vertex) {
                    alive.insert(edge.tail());
                } else {
                    alive.remove(&edge.tail());
                }

                data.vertices[edge.tail() as usize].predecessor = Some(vertex);
                data.vertices[edge.tail() as usize].weight = Some(alt_weight);
                data.queue
                    .push(DijkstraQueueElement::new(alt_weight, edge.tail()));
            }
        });

        alive.remove(&vertex);
    }

    (shortcuts, edges)
}
