use std::{fs::File, io::BufReader, path::PathBuf};

use clap::{Parser, ValueEnum};
use faster_paths::{
    search::{ch::contracted_graph::ContractedGraph, hl::hub_graph::HubGraph, PathFinding},
    utility::benchmark,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    in_file: PathBuf,
    /// Type of the input file (ch, hl, fmi)
    #[arg(short, long, value_enum)]
    file_type: FileType,
    /// Infile in .fmi format
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
            let reader = BufReader::new(File::open(&args.in_file).unwrap());
            let contractes_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
            Box::new(contractes_graph)
        }
        FileType::HL => {
            let reader = BufReader::new(File::open(&args.in_file).unwrap());
            let hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();
            Box::new(hub_graph)
        }
        FileType::FMI => todo!(),
    };

    let average_duration = benchmark(&*pathfinder, args.number_of_benchmarks);
    println!("average duration was {:?}", average_duration);
}
