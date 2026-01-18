use crate::Reader;
use crate::errors::ErrorKind;
use crate::onestore::legacy::ExGuid;
use crate::onestore::legacy::file_node::revision_manifest::RevisionManifestListReferenceFND;
use crate::onestore::legacy::file_node::shared::ParseWithRef;
use crate::onestore::legacy::file_node::{FileNodeData, FileNodeDataRef};
use parser_macros::Parse;

#[derive(Debug, Clone, Parse)]
pub(crate) struct ObjectSpaceManifestRootFND {
    pub(crate) gosid_root: ExGuid,
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectSpaceManifestListReferenceFND {
    pub(crate) gosid: ExGuid,
    // Per [section 2.1.6](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/480f3f4d-1c13-4b58-9ee5-63919b17fb11),
    // - There is at least one revision in the list.
    // - All but the last revision must be ignored.
    pub(crate) last_revision: RevisionManifestListReferenceFND,
}

impl<'a> ParseWithRef<'a> for ObjectSpaceManifestListReferenceFND {
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> crate::errors::Result<Self> {
        if let FileNodeDataRef::ElementList(data_ref) = data_ref {
            // Validation
            for (index, item) in data_ref.file_node_sequence.iter().enumerate() {
                if index == 0 {
                    if !matches!(item.fnd, FileNodeData::ObjectSpaceManifestListStartFND(_)) {
                        return Err(
                            ErrorKind::MalformedOneStoreData(
                                "ObjectSpaceManifestListReferenceFND's list must start with a ObjectSpaceManifestListStartFND.".into()
                            ).into()
                        );
                    }
                } else if !matches!(item.fnd, FileNodeData::RevisionManifestListReferenceFND(_)) {
                    return Err(
                        ErrorKind::MalformedOneStoreData(
                            "All items following the first in an ObjectSpaceManifestListReferenceFND must be RevisionManifestListReferenceFNDs.".into()
                        ).into()
                    );
                }
            }

            let last_revision =
                data_ref
                    .file_node_sequence
                    .iter()
                    .rev()
                    .find_map(|node| match &node.fnd {
                        FileNodeData::RevisionManifestListReferenceFND(revision) => Some(revision),
                        _ => None,
                    });
            if let Some(last_revision) = last_revision {
                Ok(Self {
                    gosid: ExGuid::parse(reader)?,
                    last_revision: last_revision.clone(),
                })
            } else {
                Err(
                    ErrorKind::MalformedOneStoreData(
                        "ObjectSpaceManifestListReferenceFND must point to a list with at least one revision".into()
                    ).into()
                )
            }
        } else {
            Err(ErrorKind::MalformedOneStoreData(
                "ObjectSpaceManifestListReferenceFND must point to a list of elements".into(),
            )
            .into())
        }
    }
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct ObjectSpaceManifestListStartFND {
    gsoid: ExGuid,
}
