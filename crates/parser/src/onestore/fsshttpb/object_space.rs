use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::fsshttpb::data_element::object_group::ObjectGroupData;
use crate::fsshttpb::data_element::storage_index::{StorageIndex, StorageIndexCellMapping};
use crate::fsshttpb::packaging::OneStorePackaging;
use crate::onestore::Object;
use crate::onestore::fsshttpb::revision::Revision;
use crate::onestore::fsshttpb::revision_role::RevisionRole;
use std::collections::HashMap;

pub(crate) type GroupData<'a> = HashMap<(ExGuid, u64), &'a ObjectGroupData>;

/// A OneNote object space.
///
/// Typically this is a section's metadata or a page and its content.
///
/// See [\[MS-ONESTOR\] 2.1.4]
///
/// [\[MS-ONESTOR\] 2.1.4]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/1329433f-02a5-4e83-ab41-80d57ade38d9
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ObjectSpace {
    id: ExGuid,
    context: ExGuid,
    roots: HashMap<RevisionRole, ExGuid>,
    objects: HashMap<ExGuid, Object>,
}

impl crate::onestore::ObjectSpace for ObjectSpace {
    fn get_object(&self, id: ExGuid) -> Option<&Object> {
        self.objects.get(&id)
    }

    fn content_root(&self) -> Option<ExGuid> {
        self.roots.get(&RevisionRole::DefaultContent).copied()
    }

    fn metadata_root(&self) -> Option<ExGuid> {
        self.roots.get(&RevisionRole::Metadata).copied()
    }
}

impl<'a, 'b> ObjectSpace {
    pub(crate) fn parse(
        mapping: &'a StorageIndexCellMapping,
        storage_index: &'a StorageIndex,
        packaging: &'a OneStorePackaging,
        revision_cache: &'b mut HashMap<CellId, Revision>,
    ) -> Result<(CellId, ObjectSpace)> {
        let cell_id = mapping.cell_id;

        let context_id = cell_id.0;
        let object_space_id = cell_id.1;

        let cell_revision_id = packaging
            .data_element_package
            .find_cell_revision_id(mapping.id);

        let revision_manifest_id = packaging
            .data_element_package
            .resolve_cell_revision_manifest_id(storage_index, mapping.id)
            .or_else(|| storage_index.find_revision_mapping_by_serial(&mapping.serial));

        if revision_manifest_id.is_none() && cell_revision_id.map(|id| id.is_nil()).unwrap_or(false)
        {
            return Ok((
                cell_id,
                ObjectSpace {
                    id: object_space_id,
                    context: context_id,
                    roots: HashMap::new(),
                    objects: HashMap::new(),
                },
            ));
        }

        let revision_manifest_id = revision_manifest_id.ok_or_else(|| {
            ErrorKind::MalformedOneStoreData("no revision manifest id found".into())
        })?;

        let mut objects = HashMap::new();
        let mut roots = HashMap::new();

        let mut rev_id = Some(revision_manifest_id);

        while let Some(revision_manifest_id) = rev_id {
            let base_rev_id = Revision::parse(
                revision_manifest_id,
                context_id,
                object_space_id,
                storage_index,
                packaging,
                revision_cache,
                &mut objects,
                &mut roots,
            )?;

            rev_id = base_rev_id;
        }

        let space = ObjectSpace {
            id: object_space_id,
            context: context_id,
            roots,
            objects,
        };

        Ok((cell_id, space))
    }
}
