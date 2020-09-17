use crate::fsshttpb::data_element::value::DataElementValue;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;
use crate::types::object_types::ObjectType;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct StorageManifest {
    pub(crate) id: Guid,
    pub(crate) roots: Vec<StorageManifestRoot>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct StorageManifestRoot {
    pub(crate) root_manifest: ExGuid,
    pub(crate) cell: CellId,
}

impl DataElementValue {
    pub(crate) fn parse_storage_manifest(reader: Reader) -> DataElementValue {
        let object_header = ObjectHeader::parse_16(reader);
        assert_eq!(object_header.object_type, ObjectType::StorageManifest);

        let id = Guid::parse(reader);

        let mut roots = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse_16(reader);
            assert_eq!(object_header.object_type, ObjectType::StorageManifestRoot);

            let root_manifest = ExGuid::parse(reader);
            let cell = CellId::parse(reader);

            roots.push(StorageManifestRoot {
                root_manifest,
                cell,
            })
        }

        DataElementValue::StorageManifest(StorageManifest { id, roots })
    }
}
