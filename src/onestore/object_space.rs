use crate::fsshttpb::data_element::object_group::ObjectGroupData;
use crate::fsshttpb::data_element::storage_index::{StorageIndex, StorageIndexCellMapping};
use crate::fsshttpb::data_element::value::DataElementValue;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::object::Object;
use crate::onestore::revision_role::RevisionRole;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use std::collections::HashMap;

pub(crate) type GroupData<'a> = HashMap<(ExGuid, u64), &'a ObjectGroupData>;

#[derive(Debug)]
pub(crate) struct ObjectSpace<'a> {
    id: ExGuid,
    context: ExGuid,
    roots: HashMap<RevisionRole, ExGuid>,
    objects: HashMap<ExGuid, Object<'a>>,
}

impl<'a> ObjectSpace<'a> {
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

#[derive(Debug, Clone)]
pub(crate) struct Revision<'a> {
    objects: HashMap<ExGuid, Object<'a>>,
    roots: HashMap<RevisionRole, ExGuid>,
}

impl<'a, 'b> ObjectSpace<'a> {
    pub(crate) fn parse(
        mapping: &'a StorageIndexCellMapping,
        storage_index: &'a StorageIndex,
        packaging: &'a Packaging,
        revision_cache: &'b mut HashMap<CellId, Revision<'a>>,
    ) -> (CellId, ObjectSpace<'a>) {
        let cell_id = mapping.cell_id;

        let context_id = cell_id.0;
        let object_space_id = cell_id.1;

        let cell_manifest_id = ObjectSpace::find_cell_manifest_id(mapping.id, packaging)
            .expect("cell manifest id not found");
        let revision_manifest_id = storage_index
            .find_revision_mapping_id(cell_manifest_id)
            .expect("no revision manifest id found");

        let mut objects = HashMap::new();
        let mut roots = HashMap::new();

        let mut rev_id = Some(revision_manifest_id);

        while let Some(revision_manifest_id) = rev_id {
            let base_rev_id = Self::parse_revision(
                revision_manifest_id,
                context_id,
                object_space_id,
                storage_index,
                packaging,
                revision_cache,
                &mut objects,
                &mut roots,
            );

            rev_id = base_rev_id;
        }

        (
            cell_id,
            ObjectSpace {
                id: object_space_id,
                context: context_id,
                roots,
                objects,
            },
        )
    }

    fn parse_revision(
        revision_manifest_id: ExGuid,
        context_id: ExGuid,
        object_space_id: ExGuid,
        storage_index: &'a StorageIndex,
        packaging: &'a Packaging,
        revision_cache: &'b mut HashMap<CellId, Revision<'a>>,
        objects: &'b mut HashMap<ExGuid, Object<'a>>,
        roots: &'b mut HashMap<RevisionRole, ExGuid>,
    ) -> Option<ExGuid> {
        let revision_manifest = packaging
            .data_element_package
            .find_revision_manifest(revision_manifest_id)
            .expect("revision manifest not found");
        let base_rev = revision_manifest.base_rev_id.as_option().map(|mapping_id| {
            storage_index
                .find_revision_mapping_id(mapping_id)
                .expect("revision mapping not found")
        });

        if let Some(rev) = revision_cache.get(&CellId(context_id, revision_manifest.rev_id)) {
            roots.extend(rev.roots.iter());
            objects.extend(rev.objects.clone().into_iter());

            return base_rev;
        }

        roots.extend(
            revision_manifest
                .root_declare
                .iter()
                .map(|root| (RevisionRole::parse(root.root_id), root.object_id)),
        );

        for group_id in revision_manifest.group_references.iter() {
            Self::parse_group(context_id, *group_id, object_space_id, packaging, objects)
        }

        base_rev
    }

    fn parse_group(
        context_id: ExGuid,
        group_id: ExGuid,
        object_space_id: ExGuid,
        packaging: &'a Packaging,
        objects: &'b mut HashMap<ExGuid, Object<'a>>,
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

            let object = Object::parse(
                object_id,
                context_id,
                object_space_id,
                &group_objects,
                packaging,
            );

            objects.insert(object_id, object);
        }
    }

    fn find_cell_manifest_id(cell_manifest_id: ExGuid, packaging: &'a Packaging) -> Option<ExGuid> {
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
