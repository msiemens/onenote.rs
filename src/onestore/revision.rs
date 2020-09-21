use crate::fsshttpb::data_element::object_group::ObjectGroupData;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::object::Object;
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Revision {
    id: ExGuid,
    base: ExGuid,
    roots: Vec<(RevisionRole, ExGuid)>,
    objects: HashMap<ExGuid, Object>,
}

pub(crate) type GroupData<'a> = HashMap<(ExGuid, u64), &'a ObjectGroupData>;

impl Revision {
    pub(crate) fn base_rev_id(&self) -> Option<ExGuid> {
        if self.base.is_nil() {
            None
        } else {
            Some(self.base)
        }
    }

    pub(crate) fn base_rev<'a>(&'a self, space: &'a ObjectSpace) -> Option<&'a Revision> {
        self.base_rev_id().and_then(|id| space.revisions().get(&id))
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
        space: &'a ObjectSpace,
    ) -> Option<&'a Object> {
        let object = self.objects.get(&object_id);

        if let Some(object) = object {
            return Some(object);
        }

        // Try resolving in the base revision
        self.base_rev(space)
            .and_then(|rev| rev.resolve_object(object_id, space))
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

        let mut objects = HashMap::new();

        for id in revision_manifest.group_references.iter() {
            Self::parse_group(&mut objects, *id, object_space_id, packaging)
        }

        (
            id,
            Revision {
                id,
                base,
                roots,
                objects,
            },
        )
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
            assert_eq!(group.declarations.len(), group.objects.len());

            let object = Object::parse(object_id, object_space_id, &group_objects, packaging);

            objects.insert(object_id, object);
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
