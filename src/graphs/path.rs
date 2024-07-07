use std::{fs::File, io::BufReader, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{graph_factory::GraphFactory, Graph, VertexId, Weight};
use crate::{
    ch::directed_contracted_graph::DirectedContractedGraph, classical_search::dijkstra::Dijkstra,
    hl::directed_hub_graph::DirectedHubGraph,
};

/// Represents a request for finding a shortest path in a graph.
///
/// This struct is used to encapsulate the information required to find a path
/// from a source vertex to a target vertex in a graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathRequest {
    source: VertexId,
    target: VertexId,
}

impl ShortestPathRequest {
    pub fn new(source: VertexId, target: VertexId) -> Option<ShortestPathRequest> {
        if source == target {
            return None;
        }

        Some(ShortestPathRequest { source, target })
    }

    pub fn source(&self) -> VertexId {
        self.source
    }

    pub fn target(&self) -> VertexId {
        self.target
    }
}

/// Represents a path in a graph.
///
/// This struct encapsulates the vertices that form a path in the graph and the
/// total weight associated with traversing this path.
#[derive(Clone, Serialize, Deserialize)]
pub struct Path {
    pub vertices: Vec<VertexId>,
    pub weight: Weight,
}

/// Represents a request for validating a shortest path in a graph.
///
/// This struct is used to encapsulate a shortest path request along with the
/// weight of a shortest path, if there exists one.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathTestCase {
    pub request: ShortestPathRequest,
    pub weight: Option<Weight>,
}

/// Represents a request for validating a shortest path in a graph.
///
/// This struct is used to encapsulate a shortest path request along with the
/// weight of a shortest path, if there exists one.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathTestCaseC {
    pub source: u32,
    pub target: u32,
    pub weight: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortestPathTestTimingResult {
    pub test_case: ShortestPathTestCase,
    pub timing_in_seconds: f64,
}

pub trait PathFinding: Send + Sync {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path>;

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight>;

    fn number_of_vertices(&self) -> u32;
}

pub trait PathFindingWithInternalState {
    fn shortest_path(&mut self, path_request: &ShortestPathRequest) -> Option<Path>;

    fn shortest_path_weight(&mut self, path_request: &ShortestPathRequest) -> Option<Weight>;

    fn number_of_vertices(&self) -> u32;
}

pub fn read_pathfinder(file: &PathBuf) -> Option<Box<dyn PathFinding>> {
    let pathfinder_string = file.to_str().unwrap();
    if pathfinder_string.ends_with(".gr") {
        let graph = GraphFactory::from_gr_file(file);
        let dijkstra = Dijkstra {
            graph: Box::new(graph),
        };
        return Some(Box::new(dijkstra));
    }
    if pathfinder_string.ends_with(".fmi") {
        let graph = GraphFactory::from_fmi_file(file);
        let dijkstra = Dijkstra {
            graph: Box::new(graph),
        };
        return Some(Box::new(dijkstra));
    }

    let reader = BufReader::new(File::open(file).unwrap());
    if pathfinder_string.ends_with(".di_ch_bincode") {
        let contracted_graph: DirectedContractedGraph = bincode::deserialize_from(reader).unwrap();
        return Some(Box::new(contracted_graph));
    }

    if pathfinder_string.ends_with(".di_hl_bincode") {
        let hub_graph: DirectedHubGraph = bincode::deserialize_from(reader).unwrap();
        Some(Box::new(hub_graph));
    }

    None
}
