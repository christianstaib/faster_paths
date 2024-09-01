use std::{fs::File, io::BufReader, path::Path};

use clap::ValueEnum;
use graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph};
use search::{ch::contracted_graph::ContractedGraph, hl::hub_graph::HubGraph, PathFinding};
use utility::get_progressspinner;

pub mod graphs;
pub mod search;
pub mod utility;

#[derive(Debug, ValueEnum, Clone)]
pub enum FileType {
    CH,
    HL,
    FMI,
}

pub fn reading_pathfinder(path: &Path, file_type: &FileType) -> Box<dyn PathFinding> {
    let pathfinder: Box<dyn PathFinding> = match file_type {
        FileType::CH => {
            let spinner = get_progressspinner("Reading contracted graph");
            let reader = BufReader::new(File::open(path).unwrap());
            let contractes_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
            spinner.finish_and_clear();
            Box::new(contractes_graph)
        }
        FileType::HL => {
            let spinner = get_progressspinner("Reading hub graph");
            let reader = BufReader::new(File::open(path).unwrap());
            let hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();
            spinner.finish_and_clear();
            Box::new(hub_graph)
        }
        FileType::FMI => {
            let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(path);
            Box::new(graph)
        }
    };
    pathfinder
}
