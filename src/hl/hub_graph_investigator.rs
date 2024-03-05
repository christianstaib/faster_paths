use super::hub_graph::HubGraph;

pub struct HubGraphInvestigator {}
impl HubGraphInvestigator {
    pub fn get_avg_label_size(hub_graph: &HubGraph) -> f32 {
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
}
