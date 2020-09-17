use crate::fsshttpb::data_element::value::DataElementValue;
use crate::types::binary_item::BinaryItem;
use crate::types::object_types::ObjectType;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;

impl DataElementValue {
    pub(crate) fn parse_object_data_blob(reader: Reader) -> DataElementValue {
        let object_header = ObjectHeader::parse(reader);
        assert_eq!(object_header.object_type, ObjectType::ObjectDataBlob);

        let data = BinaryItem::parse(reader);

        assert_eq!(ObjectHeader::parse_end_8(reader), ObjectType::DataElement);

        DataElementValue::ObjectDataBlob(data.value())
    }
}
