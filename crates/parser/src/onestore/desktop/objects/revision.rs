//! Parses a OneStore revision.
//!
//! See [MS-ONESTORE 2.1.9](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/90101e91-2f7f-4753-9332-31bed5b5c49d).

use super::object_group_list::ObjectGroupList;
use crate::errors::{Error, ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::desktop::file_node::FileNodeData;
use crate::onestore::desktop::file_structure::FileNodeDataIterator;
use crate::onestore::desktop::objects::global_id_table::GlobalIdTable;
use crate::onestore::desktop::objects::id_mapping::IdMapping;
use crate::onestore::desktop::objects::object::Object;
use crate::onestore::desktop::objects::parse_context::ParseContext;
use crate::onestore::shared::compact_id::CompactId;
use crate::onestore::shared::object_prop_set::ObjectPropSet;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

// See [MS-ONE 2.1.8](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-one/037e31c0-4484-4a14-819a-0ddece2cacbc)
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub(crate) enum RootRole {
    DefaultContent,
    MetadataRoot,
    VersionMetadataRoot,
}

impl TryFrom<u32> for RootRole {
    type Error = Error;
    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::DefaultContent),
            2 => Ok(Self::MetadataRoot),
            4 => Ok(Self::VersionMetadataRoot),
            other => Err(onestore_parse_error!("Invalid root role: {}", other).into()),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Revision {
    pub(crate) id: ExGuid,
    pub(crate) parent_id: ExGuid,
    pub(crate) context: Option<ExGuid>,
    pub(crate) role: u32,
    pub(crate) roots: HashMap<RootRole, ExGuid>,
    pub(crate) objects: HashMap<ExGuid, crate::onestore::Object>,
    pub(crate) id_map: IdMapping,
}

pub(crate) fn try_parse<'a>(
    iterator: &mut FileNodeDataIterator<'a>,
    context: &'a ParseContext<'a>,
    revisions: &HashMap<ExGuid, Revision>,
) -> Result<Option<Revision>> {
    let next = iterator.peek();

    match next {
        Some(
            FileNodeData::RevisionManifestStart4FND(_)
            | FileNodeData::RevisionManifestStart6FND(_)
            | FileNodeData::RevisionManifestStart7FND(_),
        ) => Ok(Some(parse(iterator, context, revisions)?)),
        _ => Ok(None),
    }
}

fn parse<'a>(
    iterator: &mut FileNodeDataIterator<'a>,
    context: &'a ParseContext<'a>,
    revisions: &HashMap<ExGuid, Revision>,
) -> Result<Revision> {
    macro_rules! iterator_skip_if_matching {
        ($iterator:expr, $match_condition:pat) => {
            if matches!($iterator.peek(), $match_condition) {
                $iterator.next();
            }
        };
    }

    let start = iterator.next();
    let (id, parent_id, role, revision_context): (ExGuid, ExGuid, u32, Option<ExGuid>) = match start
    {
        Some(FileNodeData::RevisionManifestStart4FND(data)) => (
            data.rid.into(),
            data.rid_dependent.into(),
            data.revision_role,
            None,
        ),
        Some(FileNodeData::RevisionManifestStart6FND(data)) => (
            data.rid.into(),
            data.rid_dependent.into(),
            data.revision_role,
            None,
        ),
        Some(FileNodeData::RevisionManifestStart7FND(data)) => (
            data.base.rid.into(),
            data.base.rid_dependent.into(),
            data.base.revision_role,
            Some(data.gctxid.into()),
        ),
        _ => {
            return Err(
                onestore_parse_error!("Invalid start node for revision: {:?}", start).into(),
            );
        }
    };

    let mut revision_id_map = if parent_id.is_nil() {
        IdMapping::new()
    } else {
        let parent = revisions.get(&parent_id).ok_or_else(|| {
            ErrorKind::MalformedOneStoreData(
                format!("Revision {id:?} depends on undeclared revision {parent_id:?}").into(),
            )
        })?;
        parent.id_map.clone()
    };
    let mut roots = HashMap::new();
    let mut objects = HashMap::new();

    let parent_id_map = (!parent_id.is_nil())
        .then(|| revisions.get(&parent_id).map(|parent| &parent.id_map))
        .flatten();
    let mut global_id_tables: Vec<GlobalIdTable> = Vec::new();

    let mut last_index = iterator.get_index();
    while let Some(current) = iterator.peek() {
        if let FileNodeData::RevisionManifestEndFND = current {
            iterator.next();
            break;
        } else if let Some(()) = ObjectGroupList::try_parse_into(iterator, context, &mut objects)? {
            // Used for reference counting, which does not affect the materialized object state.
            iterator_skip_if_matching!(
                iterator,
                Some(FileNodeData::ObjectInfoDependencyOverridesFND(_))
            );
        } else if let Some(global_id_table) = GlobalIdTable::try_parse(iterator, parent_id_map)? {
            revision_id_map.merge(&global_id_table.id_map);

            // In .onetoc2 files, objects can directly follow GlobalIdTables.
            let parse_context = context.with_id_table(&global_id_table);
            iterator_skip_if_matching!(
                iterator,
                Some(FileNodeData::DataSignatureGroupDefinitionFND(_))
            );

            loop {
                if let Some(object) = Object::try_parse(iterator, &parse_context)? {
                    let id = global_id_table.resolve_id(&object.compact_id)?;
                    objects.insert(id, object.data);
                } else {
                    let revision = match iterator.peek() {
                        Some(FileNodeData::ObjectRevisionWithRefCountFNDX(rev)) => {
                            Some((rev.oid(), rev.property_set().clone()))
                        }
                        Some(FileNodeData::ObjectRevisionWithRefCount2FNDX(rev)) => {
                            Some((rev.oid(), rev.property_set().clone()))
                        }
                        _ => None,
                    };

                    match revision {
                        Some((oid, props)) => {
                            iterator.next();
                            apply_object_revision(
                                &oid,
                                props,
                                &global_id_table,
                                &parse_context,
                                parent_id,
                                revisions,
                                &mut objects,
                            )?;
                        }
                        None => break,
                    }
                }

                iterator_skip_if_matching!(
                    iterator,
                    Some(FileNodeData::ObjectInfoDependencyOverridesFND(_))
                );
            }

            global_id_tables.push(global_id_table);
        } else if let FileNodeData::RootObjectReference3FND(object_reference) = current {
            iterator.next();
            roots.insert(
                object_reference.root_role.try_into()?,
                object_reference.oid_root.into(),
            );
        } else if let FileNodeData::RootObjectReference2FNDX(object_reference) = current {
            iterator.next();
            let oid_root = global_id_tables
                .last()
                .ok_or_else(|| {
                    onestore_parse_error!(
                        "Unable to resolve RootObjectReference2FNDX ID: no global ID table found"
                    )
                })?
                .resolve_id(&object_reference.oid_root)?;
            roots.insert(object_reference.root_role.try_into()?, oid_root);
        } else if let FileNodeData::DataSignatureGroupDefinitionFND(_) = current {
            iterator.next();
        } else {
            return Err(
                onestore_parse_error!("Unexpected node (parsing Revision): {:?}", current).into(),
            );
        }

        let current_index = iterator.get_index();
        if current_index == last_index {
            return Err(onestore_parse_error!(
                "Parser did not advance while parsing Revision entry: {:?}",
                current
            )
            .into());
        }
        last_index = current_index;
    }

    Ok(Revision {
        id,
        parent_id,
        context: revision_context,
        role,
        roots,
        objects,
        id_map: revision_id_map,
    })
}

fn apply_object_revision(
    oid: &CompactId,
    props: ObjectPropSet,
    global_id_table: &GlobalIdTable,
    parse_context: &ParseContext,
    parent_id: ExGuid,
    revisions: &HashMap<ExGuid, Revision>,
    objects: &mut HashMap<ExGuid, crate::onestore::Object>,
) -> Result<()> {
    let id = global_id_table.resolve_id(oid)?;

    let jc_id = objects
        .get(&id)
        .or_else(|| find_dependency_object(revisions, parent_id, id))
        .map(|object| object.jc_id)
        .ok_or_else(|| {
            ErrorKind::MalformedOneStoreData(
                format!("Object revision {id:?} has no declaration in its dependency chain").into(),
            )
        })?;

    objects.insert(
        id,
        crate::onestore::Object {
            context_id: parse_context.context_id,
            jc_id,
            props,
            file_data: None,
            mapping: parse_context.id_map.clone(),
        },
    );

    Ok(())
}

fn find_dependency_object(
    revisions: &HashMap<ExGuid, Revision>,
    mut revision_id: ExGuid,
    object_id: ExGuid,
) -> Option<&crate::onestore::Object> {
    while !revision_id.is_nil() {
        let revision = revisions.get(&revision_id)?;
        if let Some(object) = revision.objects.get(&object_id) {
            return Some(object);
        }
        revision_id = revision.parent_id;
    }
    None
}
