use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    usize,
};

use indicatif::ProgressIterator;
use serde::{Deserialize, Serialize};

use crate::{
    dijkstra_data::{dijkstra_data_map::DijkstraDataHashMap, DijkstraData},
    graphs::{
        edge::{DirectedTaillessWeightedEdge, DirectedWeightedEdge},
        path::{Path, PathFinding, ShortestPathRequest},
        vec_graph::VecGraph,
        Graph, VertexId, Weight,
    },
    queue::DijkstraQueueElement,
    simple_algorithms::bidirectional_helpers::path_from_bidirectional_search,
};

use super::shortcut_replacer::{slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer};

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub upward_graph: VecGraph,
    pub downward_graph: VecGraph,
    pub number_of_vertices: u32,
    pub shortcut_replacer: SlowShortcutReplacer,
    pub levels: Vec<Vec<u32>>,
}

impl PathFinding for ContractedGraph {
    fn shortest_path(&self, route_request: &ShortestPathRequest) -> Option<Path> {
        let (meeting_vertex, weight, forward, backward) = self.get_data(route_request);
        if weight == u32::MAX {
            return None;
        }
        let path = path_from_bidirectional_search(meeting_vertex, &forward, &backward)?;
        let path = self.shortcut_replacer.replace_shortcuts(&path);
        Some(path)
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let (_, weight, _, _) = self.get_data(path_request);
        if weight == u32::MAX {
            return None;
        }
        Some(weight)
    }
}

impl ContractedGraph {
    pub fn get_data(
        &self,
        request: &ShortestPathRequest,
    ) -> (
        VertexId,
        Weight,
        Box<dyn DijkstraData>,
        Box<dyn DijkstraData>,
    ) {
        let number_of_vertices = self.upward_graph.number_of_vertices() as usize;
        let forward_data = DijkstraDataHashMap::new(number_of_vertices, request.source());
        let backward_data = DijkstraDataHashMap::new(number_of_vertices, request.target());

        let mut forward_data: Box<dyn DijkstraData> = Box::new(forward_data);
        let mut backward_data: Box<dyn DijkstraData> = Box::new(backward_data);

        let mut meeting_weight = u32::MAX;
        let mut meeting_vertex = u32::MAX;

        let mut f = 0;
        let mut b = 0;

        while (!forward_data.is_empty() && (f < meeting_weight))
            || (!backward_data.is_empty() && (b < meeting_weight))
        {
            if f < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = forward_data.pop() {
                    let forward_weight = forward_data.get_vertex_entry(vertex).weight.unwrap();
                    f = std::cmp::max(f, forward_weight);

                    let mut stall = false;
                    for in_edge in self.downward_graph.out_edges(vertex) {
                        if let Some(predecessor_weight) =
                            forward_data.get_vertex_entry(in_edge.head()).weight
                        {
                            if predecessor_weight + in_edge.weight() < forward_weight {
                                stall = true;
                                break;
                            }
                        }
                    }

                    if !stall {
                        if let Some(backward_weight) = backward_data.get_vertex_entry(vertex).weight
                        {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                        self.upward_graph.out_edges(vertex).for_each(|edge| {
                            forward_data.update(vertex, edge.head(), edge.weight())
                        });
                    }
                }
            }

            if b < meeting_weight {
                if let Some(DijkstraQueueElement { vertex, .. }) = backward_data.pop() {
                    let backward_weight = backward_data.get_vertex_entry(vertex).weight.unwrap();
                    b = std::cmp::max(b, backward_weight);

                    let mut stall = false;
                    for out_edge in self.upward_graph.out_edges(vertex) {
                        if let Some(predecessor_weight) =
                            backward_data.get_vertex_entry(out_edge.head()).weight
                        {
                            if predecessor_weight + out_edge.weight() < backward_weight {
                                stall = true;
                                break;
                            }
                        }
                    }

                    if !stall {
                        if let Some(forward_weight) = forward_data.get_vertex_entry(vertex).weight {
                            let weight = forward_weight + backward_weight;
                            if weight < meeting_weight {
                                meeting_weight = weight;
                                meeting_vertex = vertex;
                            }
                        }
                        self.downward_graph.out_edges(vertex).for_each(|edge| {
                            backward_data.update(vertex, edge.head(), edge.weight());
                        });
                    }
                }
            }

            if f >= meeting_weight && b >= meeting_weight {
                break;
            }
        }

        (meeting_vertex, meeting_weight, forward_data, backward_data)
    }

    pub fn from_fmi_file(path: &PathBuf) -> ContractedGraph {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines().peekable();

        // skip comment line
        while let Some(next_line) = lines.peek_mut() {
            let next_line = next_line.as_mut().expect("x");
            if next_line.starts_with('#') {
                lines.by_ref().next();
            } else {
                break;
            }
        }

        lines.by_ref().next();
        let number_of_vertices: usize = lines.by_ref().next().unwrap().unwrap().parse().unwrap();
        let number_of_edges: usize = lines.by_ref().next().unwrap().unwrap().parse().unwrap();

        let mut levels = vec![0; number_of_vertices];

        let _: Vec<_> = lines
            .by_ref()
            .take(number_of_vertices)
            .progress_count(number_of_vertices as u64)
            .map(|node_line| {
                // nodeID nodeID2 latitude longitude elevation level
                let line = node_line.unwrap();
                let mut values = line.split_whitespace();
                let vertex: u32 = values
                    .next()
                    .unwrap_or_else(|| panic!("no vertex found in line {}", line))
                    .parse()
                    .unwrap_or_else(|_| panic!("unable to parse vertex in line {}", line));
                values.next();
                values.next();
                values.next();
                values.next();
                let level: u32 = values
                    .next()
                    .unwrap_or_else(|| panic!("no vertex found in line {}", line))
                    .parse()
                    .unwrap_or_else(|_| panic!("unable to parse vertex in line {}", line));

                levels[vertex as usize] = level;
            })
            .collect();

        let edges: Vec<_> = lines
            .by_ref()
            .take(number_of_edges)
            .progress_count(number_of_edges as u64)
            .filter_map(|edge_line| {
                // srcIDX trgIDX cost type maxspeed
                let line = edge_line.unwrap();
                let mut values = line.split_whitespace();
                let tail: u32 = values
                    .next()
                    .unwrap_or_else(|| panic!("no tail found in line {}", line))
                    .parse()
                    .unwrap_or_else(|_| panic!("unable to parse tail in line {}", line));
                let head: u32 = values
                    .next()
                    .unwrap_or_else(|| panic!("no head found in line {}", line))
                    .parse()
                    .unwrap_or_else(|_| panic!("unable to parse head in line {}", line));
                let weight: u32 = values
                    .next()
                    .unwrap_or_else(|| panic!("no weight found in line {}", line))
                    .parse()
                    .unwrap_or_else(|_| panic!("unable to parse weight in line {}", line));
                values.next();
                values.next();
                DirectedWeightedEdge::new(tail, head, weight)
            })
            .collect();

        let mut forward = vec![Vec::new(); number_of_vertices];
        edges
            .iter()
            .filter(|edge| levels[edge.tail() as usize] <= levels[edge.head() as usize])
            .for_each(|edge| forward[edge.tail() as usize].push(edge.tailless()));

        let mut reverse = vec![Vec::new(); number_of_vertices];
        edges
            .iter()
            .filter(|edge| levels[edge.tail() as usize] >= levels[edge.head() as usize])
            .for_each(|edge| {
                reverse[edge.head() as usize].push(DirectedTaillessWeightedEdge::new(
                    edge.tail(),
                    edge.weight(),
                ))
            });

        todo!();
        // let graph = ReversibleVecGraph {
        //     out_edges: forward,
        //     in_edges: reverse,
        // };

        // let levels = Vec::new();
        // let shortcut_replacer = SlowShortcutReplacer {
        //     shortcuts: HashMap::new(),
        // };

        // ContractedGraph {
        //     graph,
        //     shortcut_replacer,
        //     levels,
        // }
    }
}
