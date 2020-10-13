use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data_element::DataElement;
use crate::types::exguid::ExGuid;
use crate::types::object_types::ObjectType;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct RevisionManifest {
    pub(crate) rev_id: ExGuid,
    pub(crate) base_rev_id: ExGuid,
    pub(crate) root_declare: Vec<RevisionManifestRootDeclare>,
    pub(crate) group_references: Vec<ExGuid>,
}

#[derive(Debug)]
pub(crate) struct RevisionManifestRootDeclare {
    pub(crate) root_id: ExGuid,
    pub(crate) object_id: ExGuid,
}

impl RevisionManifestRootDeclare {
    fn parse(reader: Reader) -> Result<RevisionManifestRootDeclare> {
        let root_id = ExGuid::parse(reader)?;
        let object_id = ExGuid::parse(reader)?;

        Ok(RevisionManifestRootDeclare { root_id, object_id })
    }
}

impl DataElement {
    pub(crate) fn parse_revision_manifest(reader: Reader) -> Result<RevisionManifest> {
        ObjectHeader::try_parse_16(reader, ObjectType::RevisionManifest)?;

        let rev_id = ExGuid::parse(reader)?;
        let base_rev_id = ExGuid::parse(reader)?;

        let mut root_declare = vec![];
        let mut group_references = vec![];

        loop {
            if ObjectHeader::has_end_8(reader, ObjectType::DataElement)? {
                break;
            }

            let object_header = ObjectHeader::parse_16(reader)?;

            match object_header.object_type {
                ObjectType::RevisionManifestRoot => {
                    root_declare.push(RevisionManifestRootDeclare::parse(reader)?)
                }
                ObjectType::RevisionManifestGroupReference => {
                    group_references.push(ExGuid::parse(reader)?)
                }
                _ => {
                    return Err(ErrorKind::MalformedFssHttpBData(
                        format!("unexpected object type: {:x}", object_header.object_type).into(),
                    )
                    .into())
                }
            }
        }

        ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement)?;

        let manifest = RevisionManifest {
            rev_id,
            base_rev_id,
            root_declare,
            group_references,
        };

        Ok(manifest)
    }
}
