use crate::data::binary_item::BinaryItem;
use crate::data::stream_object::ObjectHeader;
use crate::types::data_element::value::DataElementValue;
use crate::Reader;

impl DataElementValue {
    pub(crate) fn parse_object_data_blob(reader: Reader) -> DataElementValue {
        let object_header = ObjectHeader::parse(reader);
        assert_eq!(object_header.object_type, 0x02);

        let data = BinaryItem::parse(reader);

        assert_eq!(ObjectHeader::parse_end_8(reader), 0x01);

        DataElementValue::ObjectDataBlob(data.value())
    }
}
