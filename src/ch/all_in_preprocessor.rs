use std::{fs::File, io::BufWriter, io::Write, time::Instant};

use ahash::{HashMap, HashMapExt, HashSet};
use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use crate::{
    ch::{
        contracted_graph::ContractedGraph, preprocessor::removing_edges_violating_level_property,
        shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer, Shortcut,
    },
    graphs::{
        edge::DirectedWeightedEdge, graph_functions::to_vec_graph, path::ShortestPathRequest,
        Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub struct AllInPrerocessor {}

impl AllInPrerocessor {
    pub fn get_ch(&mut self, mut graph: Box<dyn Graph>) -> ContractedGraph {
        println!("copying graph");
        let mut base_graph = to_vec_graph(&*graph);

        let mut shortcuts: HashMap<(VertexId, VertexId), Shortcut> = HashMap::new();
        let mut levels = Vec::new();

        let mut remaining_vertices: HashSet<VertexId> = (0..graph.number_of_vertices()).collect();

        let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
        writeln!(
            writer,
            "duration_create_shortcuts,duration_add_edges,duration_add_shortcuts,duration_remove_vertex,possible_vertex_shortcuts,vertex_shortcuts,number_of_edges,number_of_shortcuts,number_of_vertices"
        )
        .unwrap();

        let landmarks = Landmarks::new(100, &*graph);
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
                .filter(|shortcut| {
                    let request =
                        ShortestPathRequest::new(shortcut.edge.tail(), shortcut.edge.head())
                            .unwrap();
                    landmarks.landmarks.iter().all(|landmark| {
                        let upper_bound = landmark.upper_bound(&request).unwrap_or(u32::MAX);
                        shortcut.edge.weight() <= upper_bound
                    })
                })
                .filter(|shortcut| {
                    let current_weight = graph
                        .get_edge_weight(&shortcut.edge.unweighted())
                        .unwrap_or(u32::MAX);
                    shortcut.edge.weight() < current_weight
                })
                .collect();
            // let duration_create_shortcuts = start.elapsed();

            // let start = Instant::now();
            vertex_shortcuts.iter().for_each(|shortcut| {
                graph.set_edge(&shortcut.edge);
            });
            // let duration_add_edges = start.elapsed();

            let possible_shortcuts = graph.in_edges(vertex).len() * graph.out_edges(vertex).len();
            let vertex_shortcuts_len = vertex_shortcuts.len();

            // let start = Instant::now();
            // insert serial
            for shortcut in vertex_shortcuts {
                let this_key = (
                    shortcut.edge.unweighted().tail(),
                    shortcut.edge.unweighted().head(),
                );
                shortcuts.insert(this_key, shortcut);
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

        println!("generating shortcut vector");
        let shortcuts: Vec<_> = shortcuts.into_values().collect();

        println!("adding shortcuts to graph");
        for shortcut in shortcuts.iter() {
            base_graph.set_edge(&shortcut.edge);
        }

        println!("creating upward and downward_graph");
        let (upward_graph, downward_graph) =
            removing_edges_violating_level_property(&base_graph, &levels);

        println!("generatin shortcut lookup map");
        let shortcuts = shortcuts
            .iter()
            .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
            .collect();

        let shortcut_replacer = SlowShortcutReplacer::new(&shortcuts);

        ContractedGraph {
            upward_graph,
            downward_graph,
            number_of_vertices: base_graph.number_of_vertices(),
            shortcut_replacer,
            levels,
        }
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
