use std::usize;

use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::graphs::{
    edge::{DirectedEdge, DirectedWeightedEdge},
    graph::Graph,
    types::VertexId,
};

use super::{
    ch_queue::queue::CHQueue, contraction_helper::ContractionHelper,
    graph_cleaner::remove_edge_to_self,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub graph: Graph,
    pub shortcuts_map: Vec<(DirectedEdge, VertexId)>,
    pub levels: Vec<Vec<u32>>,
}

pub struct Contractor {
    graph: Graph,
    queue: CHQueue,
    levels: Vec<u32>,
}

impl Contractor {
    pub fn new(graph: &Graph) -> Self {
        let levels = vec![0; graph.in_edges.len()];
        let graph = graph.clone();
        let queue = CHQueue::new(&graph);

        Contractor {
            graph,
            queue,
            levels,
        }
    }

    pub fn get_contracted_graph(graph: &Graph) -> ContractedGraph {
        let contractor = Contractor::new(graph);
        contractor.get_graph()
    }

    pub fn get_graph(mut self) -> ContractedGraph {
        remove_edge_to_self(&mut self.graph);
        let out_edges = self.graph.out_edges.clone();
        let in_edges = self.graph.in_edges.clone();

        let shortcuts = self.contract_single_nodes();

        self.graph.out_edges = out_edges;
        self.graph.in_edges = in_edges;
        self.add_shortcuts(&shortcuts);
        self.removing_edges_violating_level_property();

        let shortcuts = shortcuts
            .iter()
            .map(|(shortcut, middle)| (shortcut.unweighted(), *middle))
            .collect();

        let max_level = self.levels.iter().max().unwrap();
        let mut levels = vec![Vec::new(); *max_level as usize + 1];

        for (vertex, level) in self.levels.iter().enumerate() {
            levels[*level as usize].push(vertex as u32);
        }

        ContractedGraph {
            graph: self.graph,
            shortcuts_map: shortcuts,
            levels,
        }
    }

    /// Generates contraction hierarchy where one node at a time is contracted.
    pub fn contract_single_nodes(&mut self) -> Vec<(DirectedWeightedEdge, VertexId)> {
        let mut shortcuts = Vec::new();

        let bar = ProgressBar::new(self.graph.in_edges.len() as u64);
        bar.set_message("contracting graph");

        let mut level = 0;
        while let Some(v) = self.queue.pop(&self.graph) {
            let shortcut_generator = ContractionHelper::new(&self.graph, 10);
            let mut this_shortcuts = shortcut_generator.generate_shortcuts(v);

            self.add_shortcuts(&this_shortcuts);
            shortcuts.append(&mut this_shortcuts);

            self.graph.remove_vertex(v);
            self.levels[v as usize] = level;

            level += 1;
            bar.inc(1);
        }
        bar.finish();

        shortcuts
    }

    /// Generates contraction hierarchy where nodes from independent node sets are contracted
    /// simultainously.
    pub fn contract_node_sets(&mut self) -> Vec<(DirectedWeightedEdge, VertexId)> {
        let mut shortcuts = Vec::new();

        let bar = ProgressBar::new(self.graph.in_edges.len() as u64);

        let mut level = 0;
        while let Some(node_set) = self.queue.pop_vec(&self.graph) {
            let shortcut_generator = ContractionHelper::new(&self.graph, 10);
            let mut this_shortcuts = node_set
                .par_iter()
                .map(|&v| shortcut_generator.generate_shortcuts(v))
                .flatten()
                .collect::<Vec<_>>();

            self.add_shortcuts(&this_shortcuts);
            shortcuts.append(&mut this_shortcuts);

            for &v in node_set.iter() {
                self.graph.remove_vertex(v);
                self.levels[v as usize] = level;
            }

            bar.inc(node_set.len() as u64);
            level += 1;
        }
        bar.finish();

        shortcuts
    }

    fn add_shortcuts(&mut self, shortcuts: &Vec<(DirectedWeightedEdge, VertexId)>) {
        shortcuts
            .iter()
            .for_each(|(edge, _)| self.graph.add_edge(&edge));
    }

    fn removing_edges_violating_level_property(&mut self) {
        self.graph
            .out_edges
            .iter_mut()
            .enumerate()
            .for_each(|(tail, edges)| {
                edges.retain(|edge| self.levels[edge.head as usize] >= self.levels[tail as usize]);
            });

        self.graph
            .in_edges
            .iter_mut()
            .enumerate()
            .for_each(|(head, edges)| {
                edges.retain(|edge| self.levels[head as usize] <= self.levels[edge.tail as usize]);
            });
    }
}
