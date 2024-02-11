use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use super::{edge::DirectedWeightedEdge, graph::Graph};

#[derive(Clone)]
pub struct GraphFactory {
    //    pub edges: Vec<DirectedWeightedEdge>,
}

impl GraphFactory {
    pub fn from_fmi_file(filename: &str) -> Graph {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        let mut lines = reader.lines();
        let number_of_nodes: usize = lines.by_ref().next().unwrap().unwrap().parse().unwrap();
        let number_of_edges: usize = lines.by_ref().next().unwrap().unwrap().parse().unwrap();

        let _: Vec<_> = lines
            .by_ref()
            .take(number_of_nodes)
            .map(|node_line| {
                // nodeID nodeID2 latitude longitude elevation
                let node_line = node_line.unwrap();
                let mut values = node_line.split_whitespace();
                values.next();
                values.next();
                values.next();
                values.next();
                values.next();
            })
            .collect();

        let edges: Vec<_> = lines
            .by_ref()
            .take(number_of_edges)
            .map(|edge_line| {
                // srcIDX trgIDX cost type maxspeed
                let line = edge_line.unwrap();
                let mut values = line.split_whitespace();
                let tail: u32 = values.next().unwrap().parse().unwrap();
                let head: u32 = values.next().unwrap().parse().unwrap();
                let cost: u32 = values.next().unwrap().parse().unwrap();
                values.next();
                values.next();
                DirectedWeightedEdge::new(tail, head, cost)
            })
            .collect();

        Graph::from_edges(&edges)
    }

    pub fn from_gr_file(filename: &str) -> Graph {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);

        let edges: Vec<_> = reader
            .lines()
            .filter_map(|edge_line| {
                // srcIDX trgIDX cost type maxspeed
                let line = edge_line.unwrap();
                let mut values = line.split_whitespace();
                let line_type = values.next().unwrap();
                if line_type != "a" {
                    return None;
                }
                let tail: u32 = values.next().unwrap().parse().unwrap();
                let head: u32 = values.next().unwrap().parse().unwrap();
                let cost: u32 = values.next().unwrap().parse().unwrap();
                Some(DirectedWeightedEdge::new(tail, head, cost))
            })
            .collect();

        Graph::from_edges(&edges)
    }
}
