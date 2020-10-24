use crate::errors::Result;
use crate::fsshttpb::data::binary_item::BinaryItem;
use crate::fsshttpb::data::object_types::ObjectType;
use crate::fsshttpb::data::stream_object::ObjectHeader;
use crate::fsshttpb::data_element::DataElement;
use crate::Reader;
use std::fmt;

pub(crate) struct ObjectDataBlob(Vec<u8>);

impl ObjectDataBlob {
    pub(crate) fn value(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for ObjectDataBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectDataBlob({} bytes)", self.0.len())
    }
}

impl DataElement {
    pub(crate) fn parse_object_data_blob(reader: Reader) -> Result<ObjectDataBlob> {
        ObjectHeader::try_parse(reader, ObjectType::ObjectDataBlob)?;

        let data = BinaryItem::parse(reader)?;

        ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement)?;

        Ok(ObjectDataBlob(data.value()))
    }
}
