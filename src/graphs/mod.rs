use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use indicatif::ProgressIterator;

pub mod vec_vec_graph;

pub type VertexId = u32;
pub type EdgeId = u32;
pub type Weight = u32;

// struct Edge {
//     pub tail: VertexId,
//     pub head: VertexId,
// }

#[derive(Clone)]
pub struct Edge {
    pub tail: VertexId,
    pub head: VertexId,
}

#[derive(Clone, Debug)]
pub struct WeightedEdge {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: Weight,
}

impl WeightedEdge {
    pub fn remove_weight(&self) -> Edge {
        Edge {
            tail: self.tail,
            head: self.head,
        }
    }

    pub fn remove_tail(&self) -> TaillessEdge {
        TaillessEdge {
            head: self.head,
            weight: self.weight,
        }
    }
}

#[derive(Clone)]
pub struct TaillessEdge {
    pub head: VertexId,
    pub weight: Weight,
}

impl TaillessEdge {
    pub fn set_tail(&self, tail: VertexId) -> WeightedEdge {
        WeightedEdge {
            tail,
            head: self.head,
            weight: self.weight,
        }
    }
}

pub trait Graph: Send + Sync {
    fn number_of_vertices(&self) -> u32;

    fn number_of_edges(&self) -> u32 {
        (0..self.number_of_vertices())
            .map(|vertex| self.edges(vertex).count() as u32)
            .sum::<u32>()
    }

    fn edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_>;

    fn get_weight(&self, edge: &Edge) -> Option<Weight>;

    fn set_weight(&mut self, edge: &Edge, weight: Option<Weight>);
}

pub fn read_edges_from_fmi_file(file: &Path) -> Vec<WeightedEdge> {
    let file = File::open(file).unwrap();
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    // skip comment lines
    while let Some(next_line) = lines.next() {
        if !next_line.unwrap().starts_with('#') {
            break;
        }
    }

    let number_of_vertices: usize = lines.next().unwrap().unwrap().parse().unwrap();
    let number_of_edges: usize = lines.next().unwrap().unwrap().parse().unwrap();

    lines
        .progress_count((number_of_vertices + number_of_edges) as u64)
        .skip(number_of_vertices)
        .take(number_of_edges)
        .map(|edge_line| {
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
            WeightedEdge { tail, head, weight }
        })
        .collect()
}
