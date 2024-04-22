use clap::Parser;
use faster_paths::{
    ch::preprocessor::Preprocessor,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::validate_path,
        path::{PathFinding, ShortestPathTestCase},
    },
};
use indicatif::ProgressIterator;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: PathBuf,
    #[arg(short, long)]
    random_pairs: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut path_finder: Vec<(&str, &dyn PathFinding, Vec<Duration>)> = Vec::new();

    let mut graph = GraphFactory::from_file(&args.graph_path);

    let preprocessor = Preprocessor::new_wittness_search();
    let contracted_graph = preprocessor.get_ch(&mut graph);

    // let dijkstra = Dijkstra::new(&graph);
    // path_finder.push(("dijkstra", &dijkstra, Vec::new()));

    path_finder.push(("ch", &contracted_graph, Vec::new()));

    let reader = BufReader::new(File::open(&args.random_pairs).unwrap());
    let random_pairs: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    for (name, path_finder, times) in path_finder.iter_mut() {
        for validation in random_pairs.iter().progress() {
            let start = Instant::now();
            let _path = path_finder.shortest_path(&validation.request);
            times.push(start.elapsed());

            let average = times.iter().sum::<Duration>() / times.len() as u32;
            println!("{:<15} {:?}", name, average);

            if let Err(err) = validate_path(&graph, validation, &_path) {
                panic!("{} wrong: {}", name, err);
            }
        }
    }

    for (name, _, times) in path_finder.iter() {
        let average = times.iter().sum::<Duration>() / times.len() as u32;
        println!("{:<15} {:?}", name, average);
    }

    let mut writer = BufWriter::new(File::create("rank.csv").unwrap());

    write!(writer, "source,taget,rank,").unwrap();
    for (name, _, _) in path_finder.iter() {
        write!(writer, "{},", name).unwrap();
    }
    writeln!(writer).unwrap();

    for i in 0..path_finder[0].2.len() {
        write!(writer, "{},", random_pairs[i].request.source()).unwrap();
        write!(writer, "{},", random_pairs[i].request.target()).unwrap();
        write!(writer, "{},", random_pairs[i].dijkstra_rank).unwrap();
        for (_, _, times) in path_finder.iter() {
            write!(writer, "{},", times[i].as_nanos()).unwrap();
        }
        writeln!(writer).unwrap();
    }
}
