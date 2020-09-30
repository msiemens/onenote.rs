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
    fn parse(reader: Reader) -> RevisionManifestRootDeclare {
        let root_id = ExGuid::parse(reader);
        let object_id = ExGuid::parse(reader);

        RevisionManifestRootDeclare { root_id, object_id }
    }
}

impl DataElement {
    pub(crate) fn parse_revision_manifest(reader: Reader) -> RevisionManifest {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, ObjectType::RevisionManifest);

        let rev_id = ExGuid::parse(reader);
        let base_rev_id = ExGuid::parse(reader);

        let mut root_declare = vec![];
        let mut group_references = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement).is_some() {
                break;
            }

            let header = ObjectHeader::parse_16(reader);

            match header.object_type {
                ObjectType::RevisionManifestRoot => {
                    root_declare.push(RevisionManifestRootDeclare::parse(reader))
                }
                ObjectType::RevisionManifestGroupReference => {
                    group_references.push(ExGuid::parse(reader))
                }
                _ => panic!("unexpected object type: 0x{:x}", header.object_type),
            }
        }

        RevisionManifest {
            rev_id,
            base_rev_id,
            root_declare,
            group_references,
        }
    }
}
