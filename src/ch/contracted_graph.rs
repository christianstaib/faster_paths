use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    usize,
};

use indicatif::ProgressIterator;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::graphs::{
    adjacency_vec_graph::AdjacencyVecGraph,
    edge::{DirectedEdge, DirectedWeightedEdge},
    Graph, VertexId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct DirectedContractedGraph {
    pub upward_graph: AdjacencyVecGraph,
    pub downward_graph: AdjacencyVecGraph,
    pub shortcuts: Vec<(DirectedEdge, VertexId)>,
    pub levels: Vec<Vec<u32>>,
}

impl DirectedContractedGraph {
    pub fn upward_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        self.upward_graph.out_edges(source)
    }

    pub fn downard_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        self.downward_graph.out_edges(source)
    }

    pub fn from_fmi_file(path: &PathBuf) -> DirectedContractedGraph {
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

        let mut levels = Vec::new();

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

                if level <= levels.len() as u32 {
                    levels.resize(level as usize + 1, Vec::new());
                }
                levels[level as usize].push(vertex);
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

        let order = levels.iter().flatten().cloned().collect_vec();

        let upward_edges = edges
            .iter()
            .filter(|edge| levels[edge.tail() as usize] <= levels[edge.head() as usize])
            .cloned()
            .collect_vec();
        let upward_graph = AdjacencyVecGraph::new(&upward_edges, &order);

        let downward_edges = edges
            .iter()
            .map(DirectedWeightedEdge::reversed)
            .filter(|edge| levels[edge.tail() as usize] <= levels[edge.head() as usize])
            .collect_vec();
        let downward_graph = AdjacencyVecGraph::new(&downward_edges, &order);

        let shortcuts = Vec::new();

        DirectedContractedGraph {
            upward_graph,
            downward_graph,
            shortcuts,
            levels,
        }
    }
}
