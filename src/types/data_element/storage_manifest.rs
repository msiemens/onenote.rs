use crate::data::cell_id::CellId;
use crate::data::exguid::ExGuid;
use crate::data::guid::Guid;
use crate::data::stream_object::ObjectHeader;
use crate::types::data_element::value::DataElementValue;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct StorageManifestRoot {
    root_manifest: ExGuid,
    cell: CellId,
}

impl DataElementValue {
    pub(crate) fn parse_storage_manifest(reader: Reader) -> DataElementValue {
        let object_header = ObjectHeader::parse_16(reader);
        assert_eq!(object_header.object_type, 0x0C);

        let id = Guid::parse(reader);

        let mut roots = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x01).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse_16(reader);
            assert_eq!(object_header.object_type, 0x07);

            let root_manifest = ExGuid::parse(reader);
            let cell = CellId::parse(reader);

            roots.push(StorageManifestRoot {
                root_manifest,
                cell,
            })
        }

        DataElementValue::StorageManifest { id, roots }
    }
}
