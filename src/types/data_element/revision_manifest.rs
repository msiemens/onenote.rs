use crate::data::exguid::ExGuid;
use crate::data::stream_object::ObjectHeader;
use crate::types::data_element::value::DataElementValue;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct RevisionManifestRootDeclare {
    root_id: ExGuid,
    object_id: ExGuid,
}

impl RevisionManifestRootDeclare {
    fn parse(reader: Reader) -> RevisionManifestRootDeclare {
        let root_id = ExGuid::parse(reader);
        let object_id = ExGuid::parse(reader);

        RevisionManifestRootDeclare { root_id, object_id }
    }
}

impl DataElementValue {
    pub(crate) fn parse_revision_manifest(reader: Reader) -> DataElementValue {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, 0x1a);

        let rev_id = ExGuid::parse(reader);
        let base_rev_id = ExGuid::parse(reader);

        let mut root_declare = vec![];
        let mut group_references = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x01).is_some() {
                break;
            }

            let header = ObjectHeader::parse_16(reader);

            match header.object_type {
                0x0A => root_declare.push(RevisionManifestRootDeclare::parse(reader)),
                0x19 => group_references.push(ExGuid::parse(reader)),
                _ => panic!("unexpected object type: 0x{:x}", header.object_type),
            }
        }

        Self::RevisionManifest {
            rev_id,
            base_rev_id,
            root_declare,
            group_references,
        }
    }
}
