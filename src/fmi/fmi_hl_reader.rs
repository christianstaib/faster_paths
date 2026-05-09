use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    str::FromStr,
};

use serde::de::DeserializeOwned;

use crate::{
    flattened_nested::FlattenedNested,
    fmi::fmi_hl_format::BINARY_HL_MAGIC,
    hub_labeling::{HubLabeling, entry::LabelEntry},
    types::{Distance, VertexId},
};

struct RootedLabelEntry<D> {
    root: VertexId,
    entry: LabelEntry<D>,
}

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

fn parse_skipped(raw: &str) -> Option<VertexId> {
    match raw {
        "None" | "-1" => None,
        vertex => vertex.parse().ok().map(VertexId::new),
    }
}

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

fn hl_from_binary_reader<D: Distance + DeserializeOwned, R: Read>(
    reader: R,
) -> Option<HubLabeling<D>> {
    bincode::deserialize_from(reader).ok()
}

fn is_binary_hl<R: BufRead>(reader: &mut R) -> io::Result<bool> {
    Ok(reader.fill_buf()?.starts_with(BINARY_HL_MAGIC))
}

pub fn read_fmi_hl<D>(file: &std::path::Path) -> Option<HubLabeling<D>>
where
    D: Distance + FromStr + DeserializeOwned,
{
    let mut reader = BufReader::new(File::open(file).ok()?);

    if is_binary_hl(&mut reader).ok()? {
        reader.consume(BINARY_HL_MAGIC.len());
        hl_from_binary_reader(reader)
    } else {
        hl_from_text_reader(reader)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, remove_file},
        io::{BufWriter, Write},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        flattened_nested::FlattenedNested,
        fmi::{
            fmi_hl_format::BINARY_HL_MAGIC, fmi_hl_reader::read_fmi_hl, fmi_hl_writer::write_fmi_hl,
        },
        hub_labeling::{HubLabeling, entry::LabelEntry},
        types::VertexId,
    };

    use super::hl_from_text_reader;

    #[test]
    fn reads_binary_hub_labeling_written_by_writer() {
        let path = std::env::temp_dir().join(format!(
            "ch-hl-roundtrip-{}-{}.bin",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let hub_labeling = HubLabeling {
            up_hub_labeling: FlattenedNested::new(&vec![
                vec![entry(0, 0, None), entry(2, 7, Some(0))],
                vec![entry(1, 0, None)],
            ]),
            down_hub_labeling: FlattenedNested::new(&vec![
                vec![entry(0, 0, None)],
                vec![entry(0, 5, Some(1)), entry(1, 0, None)],
            ]),
        };

        {
            let file = File::create(&path).unwrap();
            write_fmi_hl(BufWriter::new(file), &hub_labeling).unwrap();
        }

        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(BINARY_HL_MAGIC));

        let read = read_fmi_hl::<u32>(&path).unwrap();
        assert_eq!(
            tuples(read.up_hub_labeling.nested(0)),
            vec![(0, 0, None), (2, 7, Some(0)),]
        );
        assert_eq!(tuples(read.up_hub_labeling.nested(1)), vec![(1, 0, None)]);
        assert_eq!(
            tuples(read.down_hub_labeling.nested(1)),
            vec![(0, 5, Some(1)), (1, 0, None),]
        );

        let _ = remove_file(path);
    }

    #[test]
    fn still_reads_legacy_text_hub_labeling() {
        let mut text = Vec::new();
        writeln!(text, "2").unwrap();
        writeln!(text, "2").unwrap();
        writeln!(text, "0 2 7 0").unwrap();
        writeln!(text, "0 0 0 None").unwrap();
        writeln!(text, "1 0 5 1").unwrap();
        writeln!(text, "1 1 0 -1").unwrap();

        let read = hl_from_text_reader::<u32, _>(&text[..]).unwrap();

        assert_eq!(
            tuples(read.up_hub_labeling.nested(0)),
            vec![(0, 0, None), (2, 7, Some(0)),]
        );
        assert_eq!(
            tuples(read.down_hub_labeling.nested(1)),
            vec![(0, 5, Some(1)), (1, 0, None),]
        );
    }

    fn entry(hub: u32, distance: u32, predecessor_hub: Option<u32>) -> LabelEntry<u32> {
        LabelEntry {
            hub: VertexId::new(hub),
            distance,
            predecessor_hub: predecessor_hub.map(VertexId::new),
        }
    }

    fn tuples(label: &[LabelEntry<u32>]) -> Vec<(usize, u32, Option<usize>)> {
        label
            .iter()
            .map(|entry| {
                (
                    entry.hub.as_usize(),
                    entry.distance,
                    entry.predecessor_hub.map(VertexId::as_usize),
                )
            })
            .collect()
    }
}
