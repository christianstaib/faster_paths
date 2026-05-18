use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use serde::de::DeserializeOwned;

use crate::{
    flattened_nested::FlattenedNested,
    hub_labeling::{HubLabeling, entry::LabelEntry},
    types::{Distance, VertexId},
};

#[allow(dead_code)]
struct RootedLabelEntry<D> {
    root: VertexId,
    entry: LabelEntry<D>,
}

#[allow(dead_code)]
fn parse_label_entry<D: Distance + FromStr>(line: &str) -> Option<RootedLabelEntry<D>> {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() != 4 {
        return None;
    }

    let root = VertexId::new(parts[0].parse().ok()?);
    let hub = VertexId::new(parts[1].parse().ok()?);
    let distance = parts[2].parse().ok()?;
    let predecessor_hub = parse_skipped(parts[3]);

    Some(RootedLabelEntry {
        root,
        entry: LabelEntry {
            hub,
            distance,
            predecessor_hub,
        },
    })
}

#[allow(dead_code)]
fn parse_skipped(raw: &str) -> Option<VertexId> {
    match raw {
        "None" | "-1" => None,
        vertex => vertex.parse().ok().map(VertexId::new),
    }
}

#[allow(dead_code)]
fn read_label_entries<D: Distance + FromStr>(
    lines: &mut impl Iterator<Item = String>,
    count: usize,
) -> Option<Vec<Vec<LabelEntry<D>>>> {
    let mut labels = Vec::new();

    for _ in 0..count {
        let rooted_entry = parse_label_entry(&lines.next()?)?;

        let root = rooted_entry.root.as_usize();
        if labels.len() <= root {
            labels.resize_with(root + 1, Vec::new);
        }

        labels[root].push(rooted_entry.entry);
    }

    for label in &mut labels {
        label.sort_unstable_by_key(|entry| entry.hub);
    }

    Some(labels)
}

#[allow(dead_code)]
fn hl_from_text_reader<D: Distance + FromStr, R: Read>(reader: R) -> Option<HubLabeling<D>> {
    let mut lines = BufReader::new(reader).lines().filter_map(Result::ok);

    let num_up_entries = lines.next()?.parse().ok()?;
    let num_down_entries = lines.next()?.parse().ok()?;

    let mut up_labels = read_label_entries(&mut lines, num_up_entries)?;
    let mut down_labels = read_label_entries(&mut lines, num_down_entries)?;
    let num_vertices = up_labels.len().max(down_labels.len());

    up_labels.resize_with(num_vertices, Vec::new);
    down_labels.resize_with(num_vertices, Vec::new);

    Some(HubLabeling {
        up_hub_labeling: FlattenedNested::new(&up_labels),
        down_hub_labeling: FlattenedNested::new(&down_labels),
    })
}

pub fn read_fmi_hl<D>(file: &std::path::Path) -> Option<HubLabeling<D>>
where
    D: Distance + DeserializeOwned,
{
    let reader = BufReader::new(File::open(file).ok()?);
    let mut buffer = [];
    postcard::from_io((reader, &mut buffer))
        .ok()
        .map(|(hub_labeling, _)| hub_labeling)
}
