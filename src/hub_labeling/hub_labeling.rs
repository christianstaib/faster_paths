use crate::{flattened_nested::FlattenedNested, hub_labeling::entry::LabelEntry, types::Distance};

pub struct HubLabeling<D: Distance> {
    pub up_hub_labeling: FlattenedNested<LabelEntry<D>>,
    pub down_hub_labeling: FlattenedNested<LabelEntry<D>>,
}
