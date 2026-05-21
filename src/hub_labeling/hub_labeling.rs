use crate::{
    contraction_hierachy::ContractionHierarchy,
    data_structures::FlattenedNested,
    hub_labeling::{entry::LabelEntry, merge},
    types::Distance,
};
use serde::{Deserialize, Serialize};

/// Struct that stores a hub labeling.
///
/// For a vertex `S`, the distance of a Hub `H` in `up_hub_labeling[S]` is at least the shortest
/// path distance from `S` to `H`. Symmetrically, the distance of a Hub `H` in
/// `down_hub_labeling[S]` is at least the shortest path distance from `H` to `S`.
///
/// If there exists *a* path between two vertices `S` and `T`, then there is a common Hub `H` in
/// `up_hub_labeling[S]` and `down_hub_labeling[T]` whose summed distance `S -> H + H -> T` is
/// equal to the shortest path distance from `S` to `T`.
///
/// Each label is sorted by Hub.
///
/// Queries find the shortest path distance by choosing the common Hub with the smallest summed
/// distance, which can be done in `O(|up_hub_labeling[S]| + |down_hub_labeling[T]|)` with a
/// merge-like query ([`min_common_hub_distance`](crate::hub_labeling::entry::min_common_hub_distance)).
#[derive(Debug, Serialize, Deserialize)]
pub struct HubLabeling<D: Distance> {
    up_hub_labeling: FlattenedNested<LabelEntry<D>>,
    down_hub_labeling: FlattenedNested<LabelEntry<D>>,
}

impl<D> HubLabeling<D>
where
    D: Distance,
{
    pub fn new(
        up_hub_labeling: FlattenedNested<LabelEntry<D>>,
        down_hub_labeling: FlattenedNested<LabelEntry<D>>,
    ) -> Self {
        Self {
            up_hub_labeling,
            down_hub_labeling,
        }
    }

    pub fn try_from_contraction_hierarchy(
        contraction_hierarchy: &ContractionHierarchy<D>,
        epsilon: D,
    ) -> Option<Self> {
        merge(contraction_hierarchy, epsilon)
    }

    pub fn up_hub_labeling(&self) -> &FlattenedNested<LabelEntry<D>> {
        &self.up_hub_labeling
    }

    pub fn down_hub_labeling(&self) -> &FlattenedNested<LabelEntry<D>> {
        &self.down_hub_labeling
    }
}
