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

use super::{
    shortcut_replacer::{slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer},
    ContractedGraphTrait,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub upward_graph: VecGraph,
    pub downward_graph: VecGraph,
    pub shortcut_replacer: SlowShortcutReplacer,
    pub levels: Vec<Vec<u32>>,
}

impl ContractedGraphTrait for ContractedGraph {
    fn upward_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        self.upward_graph.out_edges(source)
    }

    fn downard_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        self.downward_graph.out_edges(source)
    }

    fn number_of_vertices(&self) -> u32 {
        std::cmp::max(
            self.upward_graph.number_of_vertices(),
            self.downward_graph.number_of_vertices(),
        )
    }

    fn number_of_edges(&self) -> u32 {
        self.upward_graph.number_of_edges() + self.downward_graph.number_of_edges()
    }
}

impl ContractedGraph {
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
