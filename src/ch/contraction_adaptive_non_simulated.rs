use std::{
    fs::File,
    io::{BufWriter, Write},
};

use ahash::{HashMap, HashMapExt, HashSet};
use indicatif::ProgressBar;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use crate::{
    ch::{
        contracted_graph::DirectedContractedGraph,
        contraction_adaptive_simulated::partition_by_levels, Shortcut,
    },
    graphs::{
        edge::{DirectedEdge, DirectedWeightedEdge},
        graph_functions::to_vec_graph,
        path::ShortestPathRequest,
        Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub struct AllInPrerocessor {}

impl AllInPrerocessor {
    pub fn get_ch(
        &mut self,
        mut graph: Box<dyn Graph>,
    ) -> (DirectedContractedGraph, HashMap<DirectedEdge, VertexId>) {
        println!("copying graph");
        let mut base_graph = to_vec_graph(&*graph);

        let mut shortcuts: HashMap<DirectedEdge, Shortcut> = HashMap::new();
        let mut levels = Vec::new();

        let mut remaining_vertices: HashSet<VertexId> = (0..graph.number_of_vertices()).collect();

        let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
        writeln!(
            writer,
            "duration_create_shortcuts,duration_add_edges,duration_add_shortcuts,duration_remove_vertex,possible_vertex_shortcuts,vertex_shortcuts,number_of_edges,number_of_shortcuts,number_of_vertices"
        )
        .unwrap();

        // println!("generating landmarks");
        // let landmarks = Landmarks::new(100, &*graph);

        println!("starting actual contraction");
        let bar = ProgressBar::new(graph.number_of_vertices() as u64);
        while let Some(vertex) = Self::get_next_vertex(&graph, &mut remaining_vertices) {
            // let start = Instant::now();
            let vertex_shortcuts: Vec<_> = graph
                .in_edges(vertex)
                .par_bridge()
                .map(|in_edge| {
                    graph
                        .out_edges(vertex)
                        .filter_map(|out_edge| {
                            let edge = DirectedWeightedEdge::new(
                                in_edge.tail(),
                                out_edge.head(),
                                in_edge.weight() + out_edge.weight(),
                            )?;
                            let shortcut = Shortcut { edge, vertex };
                            Some(shortcut)
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                // only add edges that are less expensive than currently
                .filter(|shortcut| {
                    let current_weight = graph
                        .get_edge_weight(&shortcut.edge.unweighted())
                        .unwrap_or(u32::MAX);
                    shortcut.edge.weight() < current_weight
                })
                // // only add shortcut deemed necessary by Heuristic
                // .filter(|shortcut| {
                //     let request =
                //         ShortestPathRequest::new(shortcut.edge.tail(), shortcut.edge.head())
                //             .unwrap();
                //     landmarks.landmarks.iter().all(|landmark| {
                //         let upper_bound = landmark.upper_bound(&request).unwrap_or(u32::MAX);
                //         shortcut.edge.weight() <= upper_bound
                //     })
                // })
                .collect();
            // let duration_create_shortcuts = start.elapsed();

            // let start = Instant::now();
            vertex_shortcuts.iter().for_each(|shortcut| {
                graph.set_edge(&shortcut.edge);
            });
            // let duration_add_edges = start.elapsed();

            let _possible_shortcuts = graph.in_edges(vertex).len() * graph.out_edges(vertex).len();
            let _vertex_shortcuts_len = vertex_shortcuts.len();

            // let start = Instant::now();
            // insert serial
            for shortcut in vertex_shortcuts {
                shortcuts.insert(shortcut.edge.unweighted(), shortcut);
            }
            // let duration_add_shortcuts = start.elapsed();

            // let start = Instant::now();
            graph.remove_vertex(vertex);
            // let duration_remove_vertex = start.elapsed();

            levels.push(vec![vertex]);
            // writeln!(
            //     writer,
            //     "{},{},{},{},{},{},{},{},{}",
            //     duration_create_shortcuts.as_nanos(),
            //     duration_add_edges.as_nanos(),
            //     duration_add_shortcuts.as_nanos(),
            //     duration_remove_vertex.as_nanos(),
            //     possible_shortcuts,
            //     vertex_shortcuts_len,
            //     graph.number_of_edges(),
            //     shortcuts.len(),
            //     graph.number_of_vertices() - levels.len() as u32
            // )
            // .unwrap();
            // writer.flush().unwrap();

            bar.inc(1);
        }
        bar.finish();

        println!("adding shortcuts to graph");
        for shortcut in shortcuts.values() {
            base_graph.set_edge(&shortcut.edge);
        }

        println!("creating upward and downward_graph");
        let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &levels);

        println!("generatin shortcut lookup map");
        let shortcuts = shortcuts
            .values()
            .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
            .collect();

        let directed_contracted_graph = DirectedContractedGraph {
            upward_graph,
            downward_graph,
            levels,
        };

        (directed_contracted_graph, shortcuts)
    }

    fn get_next_vertex(
        graph: &Box<dyn Graph>,
        remaining_vertices: &mut HashSet<VertexId>,
    ) -> Option<VertexId> {
        let min_vertex = *remaining_vertices
            .par_iter()
            .min_by_key(|&&vertex| graph.in_edges(vertex).len() * graph.out_edges(vertex).len())?;
        remaining_vertices.remove(&min_vertex);
        Some(min_vertex)
    }
}
