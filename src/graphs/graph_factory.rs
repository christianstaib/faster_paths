use core::panic;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use indicatif::ProgressIterator;
use rayon::iter::ParallelIterator;

use super::{edge::DirectedWeightedEdge, reversible_vec_graph::ReversibleVecGraph};

#[derive(Clone)]
pub struct GraphFactory {}

impl GraphFactory {
    pub fn from_file(path: &PathBuf) -> ReversibleVecGraph {
        let file_extension = path.extension().unwrap();
        match file_extension.to_str().unwrap() {
            "fmi" => Self::from_fmi_file(path),
            "gr" => Self::from_gr_file(path),
            _ => panic!("illegal file extension"),
        }
    }

    pub fn from_fmi_file(path: &PathBuf) -> ReversibleVecGraph {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        let mut lines = reader.lines().peekable();
        //
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

        let _: Vec<_> = lines
            .by_ref()
            .take(number_of_vertices)
            .progress_count(number_of_vertices as u64)
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
                let weight: u32 = (values
                    .next()
                    .unwrap_or_else(|| panic!("no weight found in line {}", line))
                    .parse::<f32>()
                    .unwrap_or_else(|_| panic!("unable to parse weight in line {}", line))
                    .round()
                    / 10.0) as u32;
                values.next();
                values.next();
                DirectedWeightedEdge::new(tail, head, weight)
            })
            .collect();

        ReversibleVecGraph::from_edges(&edges)
    }

    pub fn from_gr_file(path: &PathBuf) -> ReversibleVecGraph {
        let file = File::open(path).unwrap();
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
                DirectedWeightedEdge::new(tail, head, cost)
            })
            .collect();

        ReversibleVecGraph::from_edges(&edges)
    }
}
