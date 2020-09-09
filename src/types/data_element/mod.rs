use std::fmt::Debug;

use crate::data::compact_u64::CompactU64;
use crate::data::exguid::ExGuid;
use crate::data::serial_number::SerialNumber;
use crate::data::stream_object::ObjectHeader;
use crate::types::data_element::value::DataElementValue;
use crate::Reader;

mod cell_manifest;
mod data_element_fragment;
mod object_data_blob;
mod object_group;
mod revision_manifest;
mod storage_index;
mod storage_manifest;
mod value;

#[derive(Debug)]
pub(crate) struct DataElementPackage {
    header: ObjectHeader,
    elements: Vec<DataElement>,
}

impl DataElementPackage {
    pub(crate) fn parse(reader: Reader) -> DataElementPackage {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, 0x15);

        assert_eq!(reader.get_u8(), 0);

        let mut elements = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, 0x15).is_some() {
                break;
            }

            elements.push(DataElement::parse(reader));
        }

        DataElementPackage { header, elements }
    }
}

#[derive(Debug)]
pub(crate) struct DataElement {
    id: ExGuid,
    serial: SerialNumber,
    element: DataElementValue,
}

impl DataElement {
    pub(crate) fn parse(reader: Reader) -> DataElement {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, 0x01);

        let id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);
        let element_type = CompactU64::parse(reader);

        let element = DataElementValue::parse(element_type.value(), reader);

        DataElement {
            id,
            serial,
            element,
        }
    }
}
