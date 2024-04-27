use super::hub_graph::DirectedHubGraph;

pub fn get_avg_label_size(hub_graph: &DirectedHubGraph) -> f32 {
    let summed_label_size: u64 = hub_graph
        .forward_labels
        .iter()
        .map(|label| label.entries.len() as u64)
        .sum::<u64>()
        + hub_graph
            .reverse_labels
            .iter()
            .map(|label| label.entries.len() as u64)
            .sum::<u64>();
    summed_label_size as f32 / (2 * hub_graph.forward_labels.len()) as f32
}

pub fn get_label_hits(hub_graph: &DirectedHubGraph) -> Vec<u32> {
    let mut hits = vec![0; hub_graph.forward_labels.len()];

    for label in hub_graph
        .forward_labels
        .iter()
        .chain(hub_graph.reverse_labels.iter())
    {
        label
            .entries
            .iter()
            .map(|x| x.vertex)
            .for_each(|vertex| hits[vertex as usize] += 1);
    }

    hits
}
