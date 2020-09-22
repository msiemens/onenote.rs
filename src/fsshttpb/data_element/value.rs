use crate::fsshttpb::data_element::data_element_fragment::DataElementFragment;
use crate::fsshttpb::data_element::object_data_blob::ObjectDataBlob;
use crate::fsshttpb::data_element::object_group::ObjectGroup;
use crate::fsshttpb::data_element::revision_manifest::RevisionManifest;
use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::data_element::storage_manifest::StorageManifest;
use crate::types::exguid::ExGuid;
use crate::Reader;

#[derive(Debug)]
pub(crate) enum DataElementValue {
    StorageIndex(StorageIndex),
    StorageManifest(StorageManifest),
    CellManifest(ExGuid),
    RevisionManifest(RevisionManifest),
    ObjectGroup(ObjectGroup),
    DataElementFragment(DataElementFragment),
    ObjectDataBlob(ObjectDataBlob),
}

impl DataElementValue {
    pub(crate) fn parse(element_type: u64, reader: Reader) -> DataElementValue {
        match element_type {
            0x01 => Self::parse_storage_index(reader),
            0x02 => Self::parse_storage_manifest(reader),
            0x03 => Self::parse_cell_manifest(reader),
            0x04 => Self::parse_revision_manifest(reader),
            0x05 => Self::parse_object_group(reader),
            0x06 => Self::parse_data_element_fragment(reader),
            0x0A => Self::parse_object_data_blob(reader),
            _ => panic!("invalid element type: 0x{:X}", element_type),
        }
    }
}
