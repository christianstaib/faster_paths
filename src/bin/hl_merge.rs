use ch::{
    fmi::{read_fmi_ch, write_fmi_hl},
    hub_labeling::merge,
};
use clap::Parser;
use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Contraction hierarchy file
    #[arg(short, long)]
    contraction_hierarchy: PathBuf,

    /// Hub labeling File
    #[arg(short = 'l', long)]
    hub_labeling: PathBuf,
}

type DistanceType = u32;

fn main() {
    let args = Args::parse();

    let contraction_hierarchy = read_fmi_ch::<DistanceType>(&args.contraction_hierarchy).unwrap();

    let start = Instant::now();
    let hub_labeling = merge(&contraction_hierarchy);
    println!("Merging took {:?}", start.elapsed());

    let avg_label_size = hub_labeling.up_hub_labeling.num_flat() as f32
        / hub_labeling.up_hub_labeling.num_nested() as f32;
    println!("Average label size is {}", avg_label_size);

    let start = Instant::now();
    let output = File::create(args.hub_labeling).unwrap();
    write_fmi_hl(BufWriter::new(output), &hub_labeling).unwrap();
    println!("writing took {:?}", start.elapsed());
}
