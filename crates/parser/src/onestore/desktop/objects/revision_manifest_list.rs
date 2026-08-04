use std::collections::HashMap;

use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::desktop::file_node::FileNodeData;
use crate::onestore::desktop::file_node::revision_manifest::RevisionManifestListStartFND;
use crate::onestore::desktop::file_structure::FileNodeDataIterator;
use crate::onestore::desktop::objects::parse_context::ParseContext;
use crate::onestore::desktop::objects::revision;

const ACTIVE_CONTENT_ROLE: u32 = 0x1;

#[derive(Debug)]
pub(crate) struct RevisionManifestList {}

impl<'a> RevisionManifestList {
    pub(crate) fn try_parse_into(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &'a ParseContext<'a>,
        roots: &mut HashMap<crate::onestore::desktop::objects::revision::RootRole, ExGuid>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<Option<()>> {
        let next = iterator.peek();

        match next {
            Some(FileNodeData::RevisionManifestListStartFND(list_reference)) => {
                iterator.next();
                Self::parse_into(iterator, list_reference, context, roots, objects)?;
                Ok(Some(()))
            }
            _ => Ok(None),
        }
    }

    fn parse_into(
        iterator: &mut FileNodeDataIterator<'a>,
        _list_reference: &RevisionManifestListStartFND,
        context: &'a ParseContext<'a>,
        roots: &mut HashMap<crate::onestore::desktop::objects::revision::RootRole, ExGuid>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<()> {
        let mut revisions: HashMap<ExGuid, revision::Revision> = HashMap::new();
        let mut labels: HashMap<(Option<ExGuid>, u32), ExGuid> = HashMap::new();

        let mut last_index = iterator.get_index();
        while let Some(current) = iterator.peek() {
            match current {
                FileNodeData::RevisionManifestEndFND => break,
                FileNodeData::RevisionRoleDeclarationFND(data) => {
                    associate_label(
                        &revisions,
                        &mut labels,
                        data.rid.into(),
                        None,
                        data.revision_role,
                    )?;
                    iterator.next();
                }
                FileNodeData::RevisionRoleAndContextDeclarationFND(data) => {
                    associate_label(
                        &revisions,
                        &mut labels,
                        data.base.rid.into(),
                        Some(data.gctxid.into()),
                        data.base.revision_role,
                    )?;
                    iterator.next();
                }
                node => {
                    let parsed =
                        revision::try_parse(iterator, context, &revisions)?.ok_or_else(|| {
                            onestore_parse_error!(
                                "Unexpected node encountered in RevisionManifestList: {:?}",
                                node
                            )
                        })?;
                    let id = parsed.id;
                    labels.insert((parsed.context, parsed.role), id);
                    if revisions.insert(id, parsed).is_some() {
                        return Err(ErrorKind::MalformedOneStoreData(
                            format!("Duplicate revision identifier {id:?}").into(),
                        )
                        .into());
                    }
                }
            }

            let index = iterator.get_index();
            if index == last_index {
                return Err(onestore_parse_error!(
                    "Parser did not advance while parsing RevisionManifestList entry: {:?}",
                    current
                )
                .into());
            }
            last_index = index;
        }

        let active_id = labels.get(&(None, ACTIVE_CONTENT_ROLE)).ok_or_else(|| {
            ErrorKind::MalformedOneStoreData(
                "Revision manifest list has no active revision in the default context".into(),
            )
        })?;
        materialize_revision(*active_id, &revisions, roots, objects)?;

        Ok(())
    }
}

fn materialize_revision(
    active_id: ExGuid,
    revisions: &HashMap<ExGuid, revision::Revision>,
    roots: &mut HashMap<crate::onestore::desktop::objects::revision::RootRole, ExGuid>,
    objects: &mut HashMap<ExGuid, crate::onestore::Object>,
) -> Result<()> {
    let mut chain = Vec::new();
    let mut revision_id = active_id;

    while !revision_id.is_nil() {
        let revision = revisions.get(&revision_id).ok_or_else(|| {
            ErrorKind::MalformedOneStoreData(
                format!("Revision chain points to undeclared revision {revision_id:?}").into(),
            )
        })?;
        chain.push(revision);
        revision_id = revision.parent_id;
    }

    roots.clear();
    objects.clear();
    for revision in chain.into_iter().rev() {
        roots.extend(revision.roots.iter().map(|(role, id)| (*role, *id)));
        objects.extend(
            revision
                .objects
                .iter()
                .map(|(id, object)| (*id, object.clone())),
        );
    }
    Ok(())
}

fn associate_label(
    revisions: &HashMap<ExGuid, revision::Revision>,
    labels: &mut HashMap<(Option<ExGuid>, u32), ExGuid>,
    revision_id: ExGuid,
    context: Option<ExGuid>,
    role: u32,
) -> Result<()> {
    if !revisions.contains_key(&revision_id) {
        return Err(ErrorKind::MalformedOneStoreData(
            "Revision role declaration points to an undeclared revision".into(),
        )
        .into());
    }

    // A later association for the same context and role supersedes the earlier one.
    labels.insert((context, role), revision_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::onestore::desktop::objects::id_mapping::IdMapping;
    use crate::onestore::desktop::objects::revision::RootRole;
    use crate::shared::guid::Guid;
    use uuid::Uuid;

    fn id(value: u128) -> ExGuid {
        ExGuid::from_guid(Guid(Uuid::from_u128(value)), 1)
    }

    fn revision(
        id: ExGuid,
        parent_id: ExGuid,
        roots: impl IntoIterator<Item = (RootRole, ExGuid)>,
    ) -> revision::Revision {
        revision::Revision {
            id,
            parent_id,
            context: None,
            role: ACTIVE_CONTENT_ROLE,
            roots: roots.into_iter().collect(),
            objects: HashMap::new(),
            id_map: IdMapping::new(),
        }
    }

    #[test]
    fn materializes_only_the_active_dependency_chain() {
        let base_id = id(1);
        let unrelated_branch_id = id(2);
        let active_id = id(3);
        let inherited_root = id(10);
        let unrelated_root = id(11);
        let active_root = id(12);
        let revisions = HashMap::from([
            (
                base_id,
                revision(
                    base_id,
                    ExGuid::default(),
                    [(RootRole::MetadataRoot, inherited_root)],
                ),
            ),
            (
                unrelated_branch_id,
                revision(
                    unrelated_branch_id,
                    base_id,
                    [(RootRole::DefaultContent, unrelated_root)],
                ),
            ),
            (
                active_id,
                revision(
                    active_id,
                    base_id,
                    [(RootRole::DefaultContent, active_root)],
                ),
            ),
        ]);
        let mut roots = HashMap::new();
        let mut objects = HashMap::new();

        materialize_revision(active_id, &revisions, &mut roots, &mut objects).unwrap();

        assert_eq!(roots.get(&RootRole::MetadataRoot), Some(&inherited_root));
        assert_eq!(roots.get(&RootRole::DefaultContent), Some(&active_root));
        assert!(!roots.values().any(|root| *root == unrelated_root));
    }

    #[test]
    fn later_role_declaration_replaces_the_earlier_revision() {
        let first_id = id(1);
        let latest_id = id(2);
        let revisions = HashMap::from([
            (
                first_id,
                revision(first_id, ExGuid::default(), std::iter::empty()),
            ),
            (latest_id, revision(latest_id, first_id, std::iter::empty())),
        ]);
        let mut labels = HashMap::new();

        associate_label(&revisions, &mut labels, first_id, None, ACTIVE_CONTENT_ROLE).unwrap();
        associate_label(
            &revisions,
            &mut labels,
            latest_id,
            None,
            ACTIVE_CONTENT_ROLE,
        )
        .unwrap();

        assert_eq!(labels.get(&(None, ACTIVE_CONTENT_ROLE)), Some(&latest_id));
    }
}
