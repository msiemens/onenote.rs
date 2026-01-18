use std::collections::{HashMap, HashSet};

use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_node::revision_manifest::RevisionManifestListStartFND;
use crate::onestore::legacy::file_structure::FileNodeDataIterator;
use crate::onestore::legacy::objects::parse_context::ParseContext;
use crate::onestore::legacy::objects::revision::Revision;

#[derive(Debug)]
pub(crate) struct RevisionManifestList {}

impl<'a> RevisionManifestList {
    pub(crate) fn try_parse_into(
        iterator: &mut FileNodeDataIterator<'a>,
        context: &'a ParseContext<'a>,
        roots: &mut HashMap<crate::onestore::legacy::objects::revision::RootRole, ExGuid>,
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
        roots: &mut HashMap<crate::onestore::legacy::objects::revision::RootRole, ExGuid>,
        objects: &mut HashMap<ExGuid, crate::onestore::Object>,
    ) -> Result<()> {
        let mut revisions_seen: HashSet<ExGuid> = HashSet::new();

        let mut last_index = iterator.get_index();
        while let Some(current) = iterator.peek() {
            match current {
                FileNodeData::RevisionManifestEndFND => {
                    break;
                }
                FileNodeData::RevisionRoleDeclarationFND(_data) => {
                    // Ignore. If present, should always have revision_role = 0x1?
                    iterator.next();
                }
                FileNodeData::RevisionRoleAndContextDeclarationFND(data) => {
                    // Adds an additional (revision role, context) pair to some prior revision
                    // in the list.
                    // See [MS-ONESTORE 2.5.18](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/4863b0e8-fe14-49bb-a634-558c747bf0b8).
                    let base_rid: ExGuid = data.base.rid.into();
                    if revisions_seen.contains(&base_rid) {
                        iterator.next();
                        // TODO: Find a test .one file that uses this and implement:
                        log::warn!("TO-DO: Apply the new role and context to the revision");
                    } else {
                        return Err(
                            ErrorKind::MalformedOneStoreData("RevisionRoleAndContextDeclarationFND points to a non-existent revision".into()).into()
                        );
                    }
                }
                node => {
                    let revision_id = Revision::try_parse_into(iterator, context, roots, objects)?
                        .ok_or_else(|| {
                            onestore_parse_error!(
                                "Unexpected node encountered in RevisionManifestList: {:?}",
                                node
                            )
                        })?;
                    revisions_seen.insert(revision_id);
                }
            }

            let index = iterator.get_index();
            assert_ne!(index, last_index);
            last_index = index;
        }

        Ok(())
    }
}
