use crate::data::exguid::ExGuid;
use crate::data::stream_object::ObjectHeader;
use crate::types::data_element::value::DataElementValue;
use crate::Reader;

impl DataElementValue {
    pub(crate) fn parse_cell_manifest(reader: Reader) -> DataElementValue {
        let object_header = ObjectHeader::parse_16(reader);
        assert_eq!(object_header.object_type, 0x0B);

        let id = ExGuid::parse(reader);

        assert_eq!(ObjectHeader::parse_end_8(reader), 0x01);

        DataElementValue::CellManifest(id)
    }
}
