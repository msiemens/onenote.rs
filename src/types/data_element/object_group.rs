use crate::data::binary_item::BinaryItem;
use crate::data::cell_id::CellId;
use crate::data::compact_u64::CompactU64;
use crate::data::exguid::ExGuid;
use crate::data::stream_object::ObjectHeader;
use crate::types::data_element::value::DataElementValue;
use crate::Reader;

#[derive(Debug)]
pub(crate) enum ObjectGroupDeclaration {
    Object {
        object_id: ExGuid,
        partition_id: u64,
        data_size: u64,
        object_reference_count: u64,
        cell_reference_count: u64,
    },
    Blob {
        object_id: ExGuid,
        blob_id: ExGuid,
        partition_id: u64,
        object_reference_count: u64,
        cell_reference_count: u64,
    },
}

#[derive(Debug)]
pub(crate) struct ObjectGroupMetadata {
    change_frequency: ObjectChangeFrequency,
}

#[derive(Debug)]
pub(crate) enum ObjectChangeFrequency {
    Unknown = 0,
    Frequent = 1,
    Infrequent = 2,
    Independent = 3,
    Custom = 4,
}

impl ObjectChangeFrequency {
    fn parse(value: u64) -> ObjectChangeFrequency {
        match value {
            x if x == ObjectChangeFrequency::Unknown as u64 => ObjectChangeFrequency::Unknown,
            x if x == ObjectChangeFrequency::Frequent as u64 => ObjectChangeFrequency::Frequent,
            x if x == ObjectChangeFrequency::Infrequent as u64 => ObjectChangeFrequency::Infrequent,
            x if x == ObjectChangeFrequency::Independent as u64 => {
                ObjectChangeFrequency::Independent
            }
            x if x == ObjectChangeFrequency::Custom as u64 => ObjectChangeFrequency::Custom,
            x => panic!("unexpected change frequency: {}", x),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ObjectGroupData {
    Object {
        group: Vec<ExGuid>,
        cells: Vec<CellId>,
        data: Vec<u8>,
    },
    ObjectExcluded {
        group: Vec<ExGuid>,
        cells: Vec<CellId>,
        size: u64,
    },
    BlobReference {
        objects: Vec<ExGuid>,
        cells: Vec<CellId>,
        blob: ExGuid,
    },
}

impl DataElementValue {
    pub(crate) fn parse_object_group(reader: Reader) -> DataElementValue {
        let declarations = DataElementValue::parse_object_group_declarations(reader);

        let mut metadata = vec![];

        let object_header = ObjectHeader::parse(reader);
        match object_header.object_type {
            0x79 => {
                metadata = DataElementValue::parse_object_group_metadata(reader);

                let object_header = ObjectHeader::parse(reader);
                assert_eq!(object_header.object_type, 0x1E);
            }
            0x1E => {}
            _ => panic!("unexpected object type: 0x{:x}", object_header.object_type),
        }
        let objects = DataElementValue::parse_object_group_data(reader);

        assert_eq!(ObjectHeader::parse_end_8(reader), 0x01);

        DataElementValue::ObjectGroup {
            declarations,
            metadata,
            objects,
        }
    }

    fn parse_object_group_declarations(reader: Reader) -> Vec<ObjectGroupDeclaration> {
        let object_header = ObjectHeader::parse(reader);
        assert_eq!(object_header.object_type, 0x1D);

        let mut declarations = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x1D).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse(reader);
            match object_header.object_type {
                0x18 => {
                    let object_id = ExGuid::parse(reader);
                    let partition_id = CompactU64::parse(reader).value();
                    let data_size = CompactU64::parse(reader).value();
                    let object_reference_count = CompactU64::parse(reader).value();
                    let cell_reference_count = CompactU64::parse(reader).value();

                    declarations.push(ObjectGroupDeclaration::Object {
                        object_id,
                        partition_id,
                        data_size,
                        object_reference_count,
                        cell_reference_count,
                    })
                }
                0x05 => {
                    let object_id = ExGuid::parse(reader);
                    let blob_id = ExGuid::parse(reader);
                    let partition_id = CompactU64::parse(reader).value();
                    let object_reference_count = CompactU64::parse(reader).value();
                    let cell_reference_count = CompactU64::parse(reader).value();

                    declarations.push(ObjectGroupDeclaration::Blob {
                        object_id,
                        blob_id,
                        partition_id,
                        object_reference_count,
                        cell_reference_count,
                    })
                }
                _ => panic!("unexpected object type: 0x{:x}", object_header.object_type),
            }
        }

        declarations
    }

    fn parse_object_group_metadata(reader: Reader) -> Vec<ObjectGroupMetadata> {
        let mut declarations = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x79).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse_32(reader);
            assert_eq!(object_header.object_type, 0x78);

            let frequency = CompactU64::parse(reader);
            declarations.push(ObjectGroupMetadata {
                change_frequency: ObjectChangeFrequency::parse(frequency.value()),
            })
        }

        declarations
    }

    fn parse_object_group_data(reader: Reader) -> Vec<ObjectGroupData> {
        let mut objects = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x1E).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse(reader);
            match object_header.object_type {
                0x03 => {
                    let group = ExGuid::parse_array(reader);
                    let cells = CellId::parse_array(reader);
                    let size = CompactU64::parse(reader).value();

                    objects.push(ObjectGroupData::ObjectExcluded { group, cells, size })
                }
                0x16 => {
                    let group = ExGuid::parse_array(reader);
                    let cells = CellId::parse_array(reader);
                    let data = BinaryItem::parse(reader).value();

                    objects.push(ObjectGroupData::Object { group, cells, data })
                }
                0x1C => {
                    let references = ExGuid::parse_array(reader);
                    let cells = CellId::parse_array(reader);
                    let blob = ExGuid::parse(reader);

                    objects.push(ObjectGroupData::BlobReference {
                        objects: references,
                        cells,
                        blob,
                    })
                }
                _ => panic!("unexpected object type: 0x{:x}", object_header.object_type),
            }
        }

        objects
    }
}
