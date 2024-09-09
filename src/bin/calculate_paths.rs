use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{graphs::Vertex, reading_pathfinder, utility::get_progressbar, FileType};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,

    /// Path of test cases
    #[arg(short, long)]
    test_cases: PathBuf,

    /// Path of test cases
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    let reader = BufReader::new(File::open(&args.test_cases).unwrap());
    let test_cases: Vec<(Vertex, Vertex)> = serde_json::from_reader(reader).unwrap();

    let paths = test_cases
        .par_iter()
        .progress_with(get_progressbar("getting paths", test_cases.len() as u64))
        .map(|&(source, target)| pathfinder.shortest_path(source, target))
        .collect::<Vec<_>>();

    let writer = BufWriter::new(File::create(&args.paths).unwrap());
    serde_json::to_writer(writer, &paths).unwrap();
}
