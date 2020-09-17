use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::object_group::ObjectGroup;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Revision {
    id: ExGuid,
    base: ExGuid,
    roots: Vec<(RevisionRole, ExGuid)>,
    object_groups: HashMap<ExGuid, ObjectGroup>,
}

#[derive(Debug)]
pub(crate) enum RevisionRole {
    DefaultContent,
    Metadata,
    EncryptionKey,
    VersionMetadata,
}

impl Revision {
    pub(crate) fn parse(
        revision_id: ExGuid,
        object_space_id: ExGuid,
        storage_index: &StorageIndex,
        packaging: &Packaging,
    ) -> Revision {
        let revision_mapping_id = storage_index
            .find_revision_mapping_id(revision_id)
            .expect("revision mapping not found");
        let revision_manifest = packaging
            .data_element_package
            .find_revision(revision_mapping_id)
            .expect("revision manifest not found");

        let id = revision_manifest.rev_id;
        let base = revision_manifest.base_rev_id;
        let roots = revision_manifest
            .root_declare
            .iter()
            .map(|root| (RevisionRole::parse(root.root_id), root.object_id))
            .collect();

        let object_groups = revision_manifest
            .group_references
            .iter()
            .map(|id| (*id, ObjectGroup::parse(*id, object_space_id, packaging)))
            .collect();

        Revision {
            id,
            base,
            roots,
            object_groups,
        }
    }
}

impl RevisionRole {
    fn parse(id: ExGuid) -> RevisionRole {
        let guid = Guid::from_str("4A3717F8-1C14-49E7-9526-81D942DE1741").expect("invalid GUID");

        if id.guid != guid {
            panic!("invalid root declare root id")
        }

        match id {
            ExGuid { value: 1, .. } => RevisionRole::DefaultContent,
            ExGuid { value: 2, .. } => RevisionRole::Metadata,
            ExGuid { value: 3, .. } => RevisionRole::EncryptionKey,
            ExGuid { value: 4, .. } => RevisionRole::VersionMetadata,
            ExGuid { value, .. } => panic!("invalid root declare id exguid value {}", value),
        }
    }
}
