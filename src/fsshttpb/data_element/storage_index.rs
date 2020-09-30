use crate::fsshttpb::data_element::DataElement;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use crate::types::object_types::ObjectType;
use crate::types::serial_number::SerialNumber;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct StorageIndex {
    pub(crate) manifest_mappings: Vec<StorageIndexManifestMapping>,
    pub(crate) cell_mappings: HashMap<CellId, StorageIndexCellMapping>,
    pub(crate) revision_mappings: HashMap<ExGuid, StorageIndexRevisionMapping>,
}

impl StorageIndex {
    pub(crate) fn find_cell_mapping_id(&self, cell_id: CellId) -> Option<ExGuid> {
        self.cell_mappings.get(&cell_id).map(|mapping| mapping.id)
    }

    pub(crate) fn find_revision_mapping_id(&self, id: ExGuid) -> Option<ExGuid> {
        self.revision_mappings
            .get(&id)
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
    pub(crate) revision_mapping: ExGuid,
    pub(crate) serial: SerialNumber,
}

impl DataElement {
    pub(crate) fn parse_storage_index(reader: Reader) -> StorageIndex {
        let mut manifest_mappings = vec![];
        let mut cell_mappings = HashMap::new();
        let mut revision_mappings = HashMap::new();

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
                    let (id, mapping) = Self::parse_storage_index_cell_mapping(reader);

                    cell_mappings.insert(id, mapping);
                }
                ObjectType::StorageIndexRevisionMapping => {
                    let (id, mapping) = Self::parse_storage_index_revision_mapping(reader);

                    revision_mappings.insert(id, mapping);
                }
                _ => panic!("unexpected object type: 0x{:x}", object_header.object_type),
            }
        }

        StorageIndex {
            manifest_mappings,
            cell_mappings,
            revision_mappings,
        }
    }

    fn parse_storage_index_manifest_mapping(reader: Reader) -> StorageIndexManifestMapping {
        let mapping_id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexManifestMapping { mapping_id, serial }
    }

    fn parse_storage_index_cell_mapping(reader: Reader) -> (CellId, StorageIndexCellMapping) {
        let cell_id = CellId::parse(reader);
        let id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        (
            cell_id,
            StorageIndexCellMapping {
                cell_id,
                id,
                serial,
            },
        )
    }

    fn parse_storage_index_revision_mapping(
        reader: Reader,
    ) -> (ExGuid, StorageIndexRevisionMapping) {
        let id = ExGuid::parse(reader);
        let revision_mapping = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        (
            id,
            StorageIndexRevisionMapping {
                revision_mapping,
                serial,
            },
        )
    }
}
