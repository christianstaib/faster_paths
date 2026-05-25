use crate::{
    contraction_hierarchy::{ContractionHierarchy, unpack_and_concat_shortcut_paths},
    hub_labeling::{
        HubLabeling,
        entry::{min_common_hub_distance, reversed_shortcut_path},
    },
    path::{Path, Query},
    pathfinder::ShortestPathFinder,
    types::Distance,
};

pub struct HubLabelingPathfinder<'a, D: Distance> {
    contraction_hierarchy: &'a ContractionHierarchy<D>,
    hub_labeling: &'a HubLabeling<D>,
}

impl<'a, D: Distance> HubLabelingPathfinder<'a, D> {
    pub fn new(
        contraction_hierarchy: &'a ContractionHierarchy<D>,
        hub_labeling: &'a HubLabeling<D>,
    ) -> Self {
        Self {
            contraction_hierarchy,
            hub_labeling,
        }
    }
}

impl<'a, D: Distance> ShortestPathFinder for HubLabelingPathfinder<'a, D> {
    type Distance = D;

    fn path(&mut self, query: &Query) -> Option<Path<Self::Distance>> {
        let up_label = self
            .hub_labeling
            .up_hub_labeling()
            .nested(query.source.as_usize());
        let down_label = self
            .hub_labeling
            .down_hub_labeling()
            .nested(query.target.as_usize());

        let (distance, up_index, down_index) = min_common_hub_distance(up_label, down_label)?;

        let up_reversed_shortcut_path = reversed_shortcut_path(up_label, up_index)?;
        let down_reversed_shortcut_path = reversed_shortcut_path(down_label, down_index)?;

        let vertices = unpack_and_concat_shortcut_paths(
            self.contraction_hierarchy,
            &up_reversed_shortcut_path,
            &down_reversed_shortcut_path,
            self.contraction_hierarchy.num_edges() * 2,
        )?;

        Some(Path { vertices, distance })
    }

    fn distance(&mut self, query: &Query) -> Option<Self::Distance> {
        let up_label = self
            .hub_labeling
            .up_hub_labeling()
            .nested(query.source.as_usize());
        let down_label = self
            .hub_labeling
            .down_hub_labeling()
            .nested(query.target.as_usize());

        let (distance, _up_index, _down_index) = min_common_hub_distance(up_label, down_label)?;

        Some(distance)
    }
}
