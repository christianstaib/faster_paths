use crate::graphs::graph::Graph;

use super::{
    contractor::{serial_contractor::SerialContractor, Contractor},
    ContractedGraphInformation,
};

pub struct Preprocessor {}

impl Preprocessor {
    pub fn preprocess(graph: &Graph) -> ContractedGraphInformation {
        let contractor = SerialContractor::new(graph, "ECV");
        let (shortcuts, levels) = contractor.contract();

        let mut graph = graph.clone();
        for shortcut in shortcuts.iter() {
            graph.add_edge(&shortcut.edge);
        }
        graph = Self::removing_edges_violating_level_property(&graph, &levels);

        let shortcuts = shortcuts
            .iter()
            .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
            .collect();

        ContractedGraphInformation {
            graph,
            shortcuts,
            levels,
        }
    }

    fn removing_edges_violating_level_property(graph: &Graph, levels: &Vec<Vec<u32>>) -> Graph {
        let mut vertex_to_level = vec![0; graph.number_of_vertices() as usize];
        for (level, level_list) in levels.iter().enumerate() {
            for &vertex in level_list.iter() {
                vertex_to_level[vertex as usize] = level;
            }
        }
        let num_nodes = graph.number_of_vertices();
        let mut out_edges: Vec<_> = (0..num_nodes)
            .map(|tail| graph.out_edges(tail).clone())
            .collect();
        let mut in_edges: Vec<_> = (0..num_nodes)
            .map(|tail| graph.in_edges(tail).clone())
            .collect();

        out_edges.iter_mut().enumerate().for_each(|(tail, edges)| {
            edges.retain(|edge| {
                vertex_to_level[edge.head as usize] >= vertex_to_level[tail as usize]
            });
        });

        in_edges.iter_mut().enumerate().for_each(|(head, edges)| {
            edges.retain(|edge| {
                vertex_to_level[head as usize] <= vertex_to_level[edge.tail as usize]
            });
        });

        Graph::from_out_in_edges(out_edges, in_edges)
    }
}
