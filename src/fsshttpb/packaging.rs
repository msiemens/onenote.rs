use crate::errors::Result;
use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::data_element::storage_manifest::StorageManifest;
use crate::fsshttpb::data_element::value::DataElementValue;
use crate::fsshttpb::data_element::DataElementPackage;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;
use crate::types::object_types::ObjectType;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct Packaging {
    pub(crate) file_type: Guid,
    pub(crate) file: Guid,
    pub(crate) legacy_file_version: Guid,
    pub(crate) file_format: Guid,
    pub(crate) storage_index: ExGuid,
    pub(crate) cell_schema: Guid,
    pub(crate) data_element_package: DataElementPackage,
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
        assert_eq!(header.object_type, ObjectType::OneNotePackaging);

        let storage_index = ExGuid::parse(reader);
        let cell_schema = Guid::parse(reader);

        let data_element_package = DataElementPackage::parse(reader);

        assert_eq!(
            ObjectHeader::parse_end_16(reader),
            ObjectType::OneNotePackaging
        );

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

    pub(crate) fn find_storage_index(&self) -> &StorageIndex {
        self.data_element_package
            .elements
            .values()
            .find_map(|element| {
                if let DataElementValue::StorageIndex(index) = &element.element {
                    Some(index)
                } else {
                    None
                }
            })
            .expect("no storage index found")
    }

    pub(crate) fn find_storage_manifest(&self) -> &StorageManifest {
        self.data_element_package
            .elements
            .values()
            .find_map(|element| {
                if let DataElementValue::StorageManifest(manifest) = &element.element {
                    Some(manifest)
                } else {
                    None
                }
            })
            .expect("no storage manifest found")
    }
}
