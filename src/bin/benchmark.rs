use std::path::PathBuf;

use clap::Parser;
use faster_paths::{reading_pathfinder, utility::benchmark, FileType};

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

fn main() {
    let args = Args::parse();

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    let average_duration = benchmark(&*pathfinder, args.number_of_benchmarks);
    println!("average duration was {:?}", average_duration);
}
