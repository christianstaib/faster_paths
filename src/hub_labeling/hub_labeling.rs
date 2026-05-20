use crate::{
    contraction_hierachy::ContractionHierarchy,
    data_structures::FlattenedNested,
    hub_labeling::{entry::LabelEntry, merge},
    types::Distance,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HubLabeling<D: Distance> {
    pub up_hub_labeling: FlattenedNested<LabelEntry<D>>,
    pub down_hub_labeling: FlattenedNested<LabelEntry<D>>,
}

impl<D> HubLabeling<D>
where
    D: Distance,
{
    pub fn try_from_contraction_hierarchy(
        contraction_hierarchy: &ContractionHierarchy<D>,
    ) -> Option<Self> {
        merge(contraction_hierarchy)
    }
}
