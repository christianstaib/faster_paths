use std::{
    fmt::Display,
    io::{self, Write},
};

use crate::{
    hub_labeling::{HubLabeling, entry::LabelEntry},
    types::{Distance, VertexId},
};

pub fn write_fmi_hl<W: Write, D: Distance + Display>(
    mut out: W,
    hub_labeling: &HubLabeling<D>,
) -> io::Result<()> {
    writeln!(out, "{}", hub_labeling.up_hub_labeling.num_flat())?;
    writeln!(out, "{}", hub_labeling.down_hub_labeling.num_flat())?;
    write_label_entries(&mut out, &hub_labeling.up_hub_labeling)?;
    write_label_entries(&mut out, &hub_labeling.down_hub_labeling)
}

fn write_label_entries<W: Write, D: Distance + Display>(
    out: &mut W,
    labels: &crate::flattened_nested::FlattenedNested<LabelEntry<D>>,
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
    }

    Ok(())
}

fn skipped(skipped: Option<VertexId>) -> String {
    skipped.map_or_else(|| "None".to_owned(), |v| v.as_usize().to_string())
}
