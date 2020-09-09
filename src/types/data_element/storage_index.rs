use crate::data::cell_id::CellId;
use crate::data::exguid::ExGuid;
use crate::data::serial_number::SerialNumber;
use crate::Reader;
use crate::types::data_element::value::DataElementValue;
use crate::data::stream_object::ObjectHeader;

#[derive(Debug)]
pub(crate) struct StorageIndexManifestMapping {
    mapping_id: ExGuid,
    serial: SerialNumber,
}

#[derive(Debug)]
pub(crate) struct StorageIndexCellMapping {
    cell_id: CellId,
    id: ExGuid,
    serial: SerialNumber,
}

#[derive(Debug)]
pub(crate) struct StorageIndexRevisionMapping {
    id: ExGuid,
    revision_mapping: ExGuid,
    serial: SerialNumber,
}

impl DataElementValue {
    pub(crate) fn parse_storage_index(reader: Reader) -> DataElementValue {
        let mut manifest_mappings = vec![];
        let mut cell_mappings = vec![];
        let mut revision_mappings = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x01).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse_16(reader);
            match object_header.object_type {
                0x11 => manifest_mappings.push(Self::parse_storage_index_manifest_mapping(reader)),
                0x0E => cell_mappings.push(Self::parse_storage_index_cell_mapping(reader)),
                0x0D => revision_mappings.push(Self::parse_storage_index_revision_mapping(reader)),
                _ => panic!("unexpected object type: 0x{:x}", object_header.object_type)
            }
        }

        DataElementValue::StorageIndex {
            manifest_mappings,
            cell_mappings,
            revision_mappings,
        }
    }

    fn parse_storage_index_manifest_mapping(reader: Reader) -> StorageIndexManifestMapping {
        let mapping_id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexManifestMapping {
            mapping_id,
            serial
        }
    }

    fn parse_storage_index_cell_mapping(reader: Reader) -> StorageIndexCellMapping {
        let cell_id = CellId::parse(reader);
        let id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexCellMapping {
            cell_id,
            id,
            serial
        }
    }

    fn parse_storage_index_revision_mapping(reader: Reader) -> StorageIndexRevisionMapping {
        let id = ExGuid::parse(reader);
        let revision_mapping = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);

        StorageIndexRevisionMapping {
            id,
            revision_mapping,
            serial
        }
    }
}