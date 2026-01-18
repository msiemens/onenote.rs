use super::object_group_list::ObjectGroupList;
use crate::errors::{Error, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_structure::FileNodeDataIterator;
use crate::onestore::legacy::objects::global_id_table::GlobalIdTable;
use crate::onestore::legacy::objects::object::Object;
use crate::onestore::legacy::objects::parse_context::ParseContext;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

/// See [MS-ONESTORE 2.1.9](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/90101e91-2f7f-4753-9332-31bed5b5c49d)
#[derive(Debug)]
pub(crate) struct Revision {
    // pub(crate) id: ExGuid,
    // _parent_id: ExGuid,
    // pub(crate) object_groups: Vec<ObjectGroupList>,
    // pub(crate) global_id_tables: Vec<GlobalIdTable>,
    // root_objects: HashMap<RootRole, ExGuid>,
}

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

impl<'a> Revision {
    pub(crate) fn try_parse_into(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &'a ParseContext<'a>,
        roots: &mut HashMap<RootRole, ExGuid>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<Option<ExGuid>> {
        let next = iterator.peek();

        match next {
            Some(
                FileNodeData::RevisionManifestStart4FND(_)
                | FileNodeData::RevisionManifestStart6FND(_)
                | FileNodeData::RevisionManifestStart7FND(_),
            ) => Ok(Some(Self::parse_into(iterator, context, roots, objects)?)),
            _ => Ok(None),
        }
    }

    fn parse_into(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &'a ParseContext<'a>,
        roots: &mut HashMap<RootRole, ExGuid>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<ExGuid> {
        macro_rules! iterator_skip_if_matching {
            ($iterator:expr, $match_condition:pat) => {
                if matches!($iterator.peek(), $match_condition) {
                    $iterator.next();
                }
            };
        }

        let start = iterator.next();
        let (id, _parent_id): (ExGuid, ExGuid) = match start {
            Some(FileNodeData::RevisionManifestStart4FND(data)) => {
                (data.rid.into(), data.rid_dependent.into())
            }
            Some(FileNodeData::RevisionManifestStart6FND(data)) => {
                (data.rid.into(), data.rid_dependent.into())
            }
            Some(FileNodeData::RevisionManifestStart7FND(data)) => {
                (data.base.rid.into(), data.base.rid_dependent.into())
            }
            _ => {
                return Err(
                    onestore_parse_error!("Invalid start node for revision: {:?}", start).into(),
                );
            }
        };

        let mut global_id_tables: Vec<GlobalIdTable> = Vec::new();

        let mut last_index = iterator.get_index();
        while let Some(current) = iterator.peek() {
            if let FileNodeData::RevisionManifestEndFND = current {
                iterator.next();
                break;
            } else if let Some(()) = ObjectGroupList::try_parse_into(iterator, context, objects)? {
                // Skip: Used for reference counting (which we can ignore here)
                iterator_skip_if_matching!(
                    iterator,
                    Some(FileNodeData::ObjectInfoDependencyOverridesFND(_))
                );
            } else if let Some(global_id_table) = GlobalIdTable::try_parse(iterator)? {
                // In .onetoc2 files, objects can directly follow GlobalIdTables:
                let parse_context = context.with_id_table(&global_id_table);
                iterator_skip_if_matching!(
                    iterator,
                    Some(FileNodeData::DataSignatureGroupDefinitionFND(_))
                );

                while let Some(object) = Object::try_parse(iterator, &parse_context)? {
                    let id = global_id_table.resolve_id(&object.compact_id)?;
                    objects.entry(id).or_insert(object.data);

                    // Skip the reference counting object, if present
                    iterator_skip_if_matching!(
                        iterator,
                        Some(FileNodeData::ObjectInfoDependencyOverridesFND(_))
                    );
                }

                global_id_tables.push(global_id_table);
            } else if let FileNodeData::RootObjectReference3FND(object_reference) = current {
                iterator.next(); // Consume the reference

                let root_role: RootRole = object_reference.root_role.try_into()?;
                if roots.contains_key(&root_role) {
                    log::warn!("An item with role {:?} is already present", root_role);
                }

                roots
                    .entry(root_role)
                    .or_insert(object_reference.oid_root.into());
            } else if let FileNodeData::RootObjectReference2FNDX(object_reference) = current {
                // .onetoc2
                iterator.next();
                let oid_root = global_id_tables
                    .last()
                    .ok_or_else(|| {
                        onestore_parse_error!(
                            "Unable to resolve RootObjectReference2FNDX ID: no global ID table found"
                        )
                    })?
                    .resolve_id(&object_reference.oid_root)?;
                roots
                    .entry(object_reference.root_role.try_into()?)
                    .or_insert(oid_root);
            } else if let FileNodeData::DataSignatureGroupDefinitionFND(_) = current {
                // .onetoc2
                log::debug!("Ignoring DataSignatureGroupDefinitionFND");

                iterator.next();
            } else {
                return Err(onestore_parse_error!(
                    "Unexpected node (parsing Revision): {:?}",
                    current
                )
                .into());
            }

            // Prevent infinite loops
            let current_index = iterator.get_index();
            assert_ne!(last_index, current_index);
            last_index = current_index;
        }

        Ok(id)
    }
}
