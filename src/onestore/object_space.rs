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
    pub(crate) fn context(&self) -> ExGuid {
        self.context
    }

    pub(crate) fn revisions(&self) -> &HashMap<ExGuid, Revision> {
        &self.revisions
    }

    pub(crate) fn find_root_revision(&self) -> Option<&Revision> {
        self.revisions.values().find(|r| !r.roots().is_empty())
    }
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

        let cell_manifest_id = ObjectSpace::find_cell_manifest_id(mapping.id, packaging)
            .expect("cell manifest id not found");
        let revision_manifest_id = storage_index
            .find_revision_mapping_id(cell_manifest_id)
            .expect("no revision manifest id found");

        let (rev_id, revision) = Revision::parse(revision_manifest_id, object_space_id, packaging);
        let mut base_rev_mapping_id = revision.base();

        let mut revisions = HashMap::new();
        revisions.insert(rev_id, revision);

        // Resolve all base revisions
        loop {
            if base_rev_mapping_id.is_nil() {
                break;
            }

            let base_rev_manifest_id = storage_index
                .find_revision_mapping_id(base_rev_mapping_id)
                .expect("revision mapping not found");
            let (rev_id, revision) =
                Revision::parse(base_rev_manifest_id, object_space_id, packaging);
            base_rev_mapping_id = revision.base();

            revisions.insert(rev_id, revision);
        }

        (
            object_space_id,
            ObjectSpace {
                id: object_space_id,
                context,
                revisions,
            },
        )
    }

    fn find_cell_manifest_id(cell_manifest_id: ExGuid, packaging: &Packaging) -> Option<ExGuid> {
        packaging
            .data_element_package
            .elements
            .iter()
            .find_map(|element| {
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
    }
}
