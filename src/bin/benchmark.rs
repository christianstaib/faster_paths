use std::{fs::File, io::BufReader, path::PathBuf};

use clap::{Parser, ValueEnum};
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph},
    search::{ch::contracted_graph::ContractedGraph, hl::hub_graph::HubGraph, PathFinding},
    utility::benchmark,
};

/// Does a single threaded benchmark.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: PathBuf,
    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,
    /// Number of benchmarks to be run.
    #[arg(short, long)]
    number_of_benchmarks: u32,
}

#[derive(Debug, ValueEnum, Clone)]
enum FileType {
    CH,
    HL,
    FMI,
}

fn main() {
    let args = Args::parse();

    let pathfinder: Box<dyn PathFinding> = match args.file_type {
        FileType::CH => {
            let reader = BufReader::new(File::open(&args.file).unwrap());
            let contractes_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
            Box::new(contractes_graph)
        }
        FileType::HL => {
            let reader = BufReader::new(File::open(&args.file).unwrap());
            let hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();
            Box::new(hub_graph)
        }
        FileType::FMI => {
            let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(args.file.as_path());
            Box::new(graph)
        }
    };

    let average_duration = benchmark(&*pathfinder, args.number_of_benchmarks);
    println!("average duration was {:?}", average_duration);
}
