use crate::fsshttpb::data_element::storage_index::{StorageIndex, StorageIndexCellMapping};
use crate::fsshttpb::data_element::value::DataElementValue;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::revision::Revision;
use crate::types::exguid::ExGuid;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct ObjectSpace {
    id: ExGuid,
    context: ExGuid,
    revisions: HashMap<ExGuid, Revision>,
}

impl ObjectSpace {
    pub(crate) fn parse(
        mapping: &StorageIndexCellMapping,
        storage_index: &StorageIndex,
        packaging: &Packaging,
    ) -> (ExGuid, ObjectSpace) {
        let cell_id = mapping.cell_id;

        let context = cell_id.0;
        let object_space_id = cell_id.1;

        let cell_manifest_id = mapping.id;
        let revisions = ObjectSpace::find_cell_revisions(cell_manifest_id, packaging)
            .into_iter()
            .map(|rev_id| {
                (
                    rev_id,
                    Revision::parse(rev_id, object_space_id, storage_index, packaging),
                )
            })
            .collect();

        (
            object_space_id,
            ObjectSpace {
                context,
                id: object_space_id,
                revisions,
            },
        )
    }

    fn find_cell_revisions(cell_manifest_id: ExGuid, packaging: &Packaging) -> Vec<ExGuid> {
        packaging
            .data_element_package
            .elements
            .iter()
            .filter_map(|element| {
                if element.id == cell_manifest_id {
                    if let DataElementValue::CellManifest(revision_id) = &element.element {
                        Some(*revision_id)
                    } else {
                        panic!("data element is not a cell manifest")
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}
