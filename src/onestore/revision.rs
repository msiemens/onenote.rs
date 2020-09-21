use crate::fsshttpb::packaging::Packaging;
use crate::onestore::object::Object;
use crate::onestore::object_group::ObjectGroup;
use crate::onestore::OneStore;
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

impl Revision {
    pub(crate) fn base(&self) -> ExGuid {
        self.base
    }

    pub(crate) fn roots(&self) -> &[(RevisionRole, ExGuid)] {
        &self.roots
    }

    pub(crate) fn content_root(&self) -> Option<ExGuid> {
        self.roots
            .iter()
            .find(|(r, _)| *r == RevisionRole::DefaultContent)
            .map(|(_, id)| id)
            .copied()
    }

    pub(crate) fn metadata_root(&self) -> Option<ExGuid> {
        self.roots
            .iter()
            .find(|(r, _)| *r == RevisionRole::Metadata)
            .map(|(_, id)| id)
            .copied()
    }

    pub(crate) fn resolve_object<'a>(
        &'a self,
        object_id: ExGuid,
        store: &'a OneStore,
    ) -> Option<&'a Object> {
        self.object_groups.values().find_map(|group| {
            group
                .objects()
                .iter()
                .find_map(|(id, object)| if *id == object_id { Some(object) } else { None })
                .or_else(|| {
                    if self.base.is_nil() {
                        None
                    } else {
                        store
                            .find_revision(self.base)
                            .expect("base revison is missing")
                            .resolve_object(object_id, store)
                    }
                })
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum RevisionRole {
    DefaultContent,
    Metadata,
    EncryptionKey,
    VersionMetadata,
}

impl Revision {
    pub(crate) fn parse(
        revision_manifest_id: ExGuid,
        object_space_id: ExGuid,
        packaging: &Packaging,
    ) -> (ExGuid, Revision) {
        let revision_manifest = packaging
            .data_element_package
            .find_revision_manifest(revision_manifest_id)
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

        (
            id,
            Revision {
                id,
                base,
                roots,
                object_groups,
            },
        )
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
