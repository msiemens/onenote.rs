use crate::fsshttpb::data_element::object_group::ObjectGroupData;
use crate::fsshttpb::data_element::storage_index::{StorageIndex, StorageIndexCellMapping};
use crate::fsshttpb::data_element::value::DataElementValue;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::object::Object;
use crate::onestore::revision_role::RevisionRole;
use crate::types::exguid::ExGuid;
use std::collections::HashMap;

pub(crate) type GroupData<'a> = HashMap<(ExGuid, u64), &'a ObjectGroupData>;

#[derive(Debug)]
pub(crate) struct ObjectSpace {
    id: ExGuid,
    context: ExGuid,
    roots: HashMap<RevisionRole, ExGuid>,
    objects: HashMap<ExGuid, Object>,
}

impl ObjectSpace {
    pub(crate) fn get_object(&self, id: ExGuid) -> Option<&Object> {
        self.objects.get(&id)
    }

    pub(crate) fn context(&self) -> ExGuid {
        self.context
    }

    pub(crate) fn content_root(&self) -> Option<ExGuid> {
        self.roots.get(&RevisionRole::DefaultContent).copied()
    }

    pub(crate) fn metadata_root(&self) -> Option<ExGuid> {
        self.roots.get(&RevisionRole::Metadata).copied()
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

        let mut objects = HashMap::new();
        let mut roots = HashMap::new();

        Self::parse_revision(
            &mut objects,
            &mut roots,
            revision_manifest_id,
            object_space_id,
            storage_index,
            packaging,
        );

        (
            object_space_id,
            ObjectSpace {
                id: object_space_id,
                context,
                roots,
                objects,
            },
        )
    }

    fn parse_revision(
        objects: &mut HashMap<ExGuid, Object>,
        roots: &mut HashMap<RevisionRole, ExGuid>,
        revision_manifest_id: ExGuid,
        object_space_id: ExGuid,
        storage_index: &StorageIndex,
        packaging: &Packaging,
    ) {
        let revision_manifest = packaging
            .data_element_package
            .find_revision_manifest(revision_manifest_id)
            .expect("revision manifest not found");

        roots.extend(
            revision_manifest
                .root_declare
                .iter()
                .map(|root| (RevisionRole::parse(root.root_id), root.object_id)),
        );

        for group_id in revision_manifest.group_references.iter() {
            Self::parse_group(objects, *group_id, object_space_id, packaging)
        }

        let base_mapping_id = revision_manifest.base_rev_id;
        if !base_mapping_id.is_nil() {
            let base_rev_manifest_id = storage_index
                .find_revision_mapping_id(base_mapping_id)
                .expect("revision mapping not found");

            Self::parse_revision(
                objects,
                roots,
                base_rev_manifest_id,
                object_space_id,
                storage_index,
                packaging,
            );
        }
    }

    fn parse_group(
        objects: &mut HashMap<ExGuid, Object>,
        group_id: ExGuid,
        object_space_id: ExGuid,
        packaging: &Packaging,
    ) {
        let group = packaging
            .data_element_package
            .find_object_group(group_id)
            .expect("object group not found");

        let object_ids: Vec<_> = group.declarations.iter().map(|o| o.object_id()).collect();

        let group_objects: GroupData = group
            .declarations
            .iter()
            .zip(group.objects.iter())
            .map(|(decl, data)| ((decl.object_id(), decl.partition_id()), data))
            .collect();

        for object_id in object_ids {
            if objects.contains_key(&object_id) {
                continue;
            }

            assert_eq!(group.declarations.len(), group.objects.len());

            let object = Object::parse(object_id, object_space_id, &group_objects, packaging);

            objects.insert(object_id, object);
        }
    }

    fn find_cell_manifest_id(cell_manifest_id: ExGuid, packaging: &Packaging) -> Option<ExGuid> {
        packaging
            .data_element_package
            .elements
            .get(&cell_manifest_id)
            .map(|element| {
                if let DataElementValue::CellManifest(revision_id) = &element.element {
                    *revision_id
                } else {
                    panic!("data element is not a cell manifest")
                }
            })
    }
}
