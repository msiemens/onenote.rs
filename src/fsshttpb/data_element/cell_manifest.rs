use crate::errors::Result;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::fsshttpb::data::object_types::ObjectType;
use crate::fsshttpb::data::stream_object::ObjectHeader;
use crate::fsshttpb::data_element::DataElement;
use crate::Reader;

impl DataElement {
    pub(crate) fn parse_cell_manifest(reader: Reader) -> Result<ExGuid> {
        ObjectHeader::try_parse_16(reader, ObjectType::CellManifest)?;

        let id = ExGuid::parse(reader)?;

        ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement)?;

        Ok(id)
    }
}
