use crate::fsshttpb::data_element::value::DataElementValue;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use crate::types::object_types::ObjectType;
use crate::types::serial_number::SerialNumber;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct StorageIndex {
    pub(crate) manifest_mappings: Vec<StorageIndexManifestMapping>,
    pub(crate) cell_mappings: Vec<StorageIndexCellMapping>,
    pub(crate) revision_mappings: Vec<StorageIndexRevisionMapping>,
}

impl StorageIndex {
    pub(crate) fn find_cell_mapping_id(&self, cell_id: CellId) -> Option<ExGuid> {
        self.cell_mappings
            .iter()
            .find(|mapping| mapping.cell_id == cell_id)
            .map(|mapping| mapping.id)
    }

    pub(crate) fn find_revision_mapping_id(&self, id: ExGuid) -> Option<ExGuid> {
        self.revision_mappings
            .iter()
            .find(|mapping| mapping.id == id)
            .map(|mapping| mapping.revision_mapping)
    }
}

#[derive(Debug)]
pub(crate) struct StorageIndexManifestMapping {
    pub(crate) mapping_id: ExGuid,
    pub(crate) serial: SerialNumber,
}

#[derive(Debug)]
pub(crate) struct StorageIndexCellMapping {
    pub(crate) cell_id: CellId,
    pub(crate) id: ExGuid,
    pub(crate) serial: SerialNumber,
}

#[derive(Debug)]
pub(crate) struct StorageIndexRevisionMapping {
    pub(crate) id: ExGuid,
    pub(crate) revision_mapping: ExGuid,
    pub(crate) serial: SerialNumber,
}

impl DataElementValue {
    pub(crate) fn parse_storage_index(reader: Reader) -> DataElementValue {
        let mut manifest_mappings = vec![];
        let mut cell_mappings = vec![];
        let mut revision_mappings = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse_16(reader);
            match object_header.object_type {
                ObjectType::StorageIndexManifestMapping => {
                    manifest_mappings.push(Self::parse_storage_index_manifest_mapping(reader))
                }
                ObjectType::StorageIndexCellMapping => {
                    cell_mappings.push(Self::parse_storage_index_cell_mapping(reader))
                }
                ObjectType::StorageIndexRevisionMapping => {
                    revision_mappings.push(Self::parse_storage_index_revision_mapping(reader))
                }
                _ => panic!("unexpected object type: 0x{:x}", object_header.object_type),
            }
        }

        DataElementValue::StorageIndex(StorageIndex {
            manifest_mappings,
            cell_mappings,
            revision_mappings,
        })
    }

    fn parse_storage_index_manifest_mapping(reader: Reader) -> StorageIndexManifestMapping {
        let mapping_id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexManifestMapping { mapping_id, serial }
    }

    fn parse_storage_index_cell_mapping(reader: Reader) -> StorageIndexCellMapping {
        let cell_id = CellId::parse(reader);
        let id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexCellMapping {
            cell_id,
            id,
            serial,
        }
    }

    fn parse_storage_index_revision_mapping(reader: Reader) -> StorageIndexRevisionMapping {
        let id = ExGuid::parse(reader);
        let revision_mapping = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexRevisionMapping {
            id,
            revision_mapping,
            serial,
        }
    }
}
