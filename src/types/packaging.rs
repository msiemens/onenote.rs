use crate::data::exguid::ExGuid;
use crate::data::guid::Guid;
use crate::data::stream_object::ObjectHeader;
use crate::errors::Result;
use crate::Reader;
use crate::types::data_element::DataElementPackage;

#[derive(Debug)]
pub struct Packaging {
    file_type: Guid,
    file: Guid,
    legacy_file_version: Guid,
    file_format: Guid,
    storage_index: ExGuid,
    cell_schema: Guid,
    data_element_package: DataElementPackage,
}

impl Packaging {
    pub(crate) fn parse(reader: Reader) -> Result<Packaging> {
        let file_type = Guid::parse(reader);
        let file = Guid::parse(reader);
        let legacy_file_version = Guid::parse(reader);
        let file_format = Guid::parse(reader);

        assert_eq!(file, legacy_file_version);

        assert_eq!(reader.get_u32_le(), 0);

        let header = ObjectHeader::parse_32(reader);
        assert_eq!(header.object_type, 0x7a);

        let storage_index = ExGuid::parse(reader);
        let cell_schema = Guid::parse(reader);

        let data_element_package = DataElementPackage::parse(reader);

        assert_eq!(ObjectHeader::parse_end_16(reader), 0x7a);

        Ok(Packaging {
            file_type,
            file,
            legacy_file_version,
            file_format,
            storage_index,
            cell_schema,
            data_element_package,
        })
    }
}
