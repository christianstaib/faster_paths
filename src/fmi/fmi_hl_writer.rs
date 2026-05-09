use indicatif::ProgressBar;
use std::fmt::Display;
use std::io::{self, Write};

use crate::flattened_nested::FlattenedNested;
use crate::hub_labeling::HubLabeling;
use crate::hub_labeling::entry::LabelEntry;
use crate::types::{Distance, VertexId};

pub fn write_fmi_hl<W: Write, D: Distance + Display>(
    mut out: W,
    hub_labeling: &HubLabeling<D>,
) -> io::Result<()> {
    let num_up_entries = hub_labeling.up_hub_labeling.num_flat();
    let num_down_entries = hub_labeling.down_hub_labeling.num_flat();
    let total_entries = num_up_entries + num_down_entries;

    let progress = ProgressBar::new(total_entries as u64);

    writeln!(out, "{}", num_up_entries)?;
    writeln!(out, "{}", num_down_entries)?;

    write_label_entries(&mut out, &hub_labeling.up_hub_labeling, &progress)?;
    write_label_entries(&mut out, &hub_labeling.down_hub_labeling, &progress)?;

    progress.finish();

    Ok(())
}

fn write_label_entries<W: Write, D: Distance + Display>(
    out: &mut W,
    labels: &FlattenedNested<LabelEntry<D>>,
    progress: &ProgressBar,
) -> io::Result<()> {
    for root in 0..labels.num_nested() {
        for entry in labels.nested(root) {
            writeln!(
                out,
                "{} {} {} {}",
                root,
                entry.hub.as_usize(),
                entry.distance,
                skipped(entry.predecessor_hub),
            )?;
        }
        progress.inc(labels.nested(root).len() as u64);
    }

    Ok(())
}

fn skipped(skipped: Option<VertexId>) -> String {
    skipped.map_or_else(|| "None".to_owned(), |v| v.as_usize().to_string())
}

