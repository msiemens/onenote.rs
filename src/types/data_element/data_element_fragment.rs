use crate::Reader;
use crate::types::data_element::value::DataElementValue;
use crate::data::exguid::ExGuid;
use crate::data::compact_u64::CompactU64;
use crate::data::stream_object::ObjectHeader;

#[derive(Debug)]
pub(crate) struct DataElementFragmentChunkReference {
    offset: u64,
    length: u64,
}

impl DataElementValue {
    pub(crate) fn parse_data_element_fragment(reader: Reader) -> DataElementValue {
        let object_header = ObjectHeader::parse(reader);
        assert_eq!(object_header.object_type, 0x06A);

        let id = ExGuid::parse(reader);
        let size = CompactU64::parse(reader).value();
        let offset = CompactU64::parse(reader).value();
        let length = CompactU64::parse(reader).value();

        let data = reader.bytes()[0..(size as usize)].to_vec();
        reader.advance(size as usize);

        let chunk_reference = DataElementFragmentChunkReference { offset, length };
        DataElementValue::DataElementFragment {
            id,
            size,
            chunk_reference,
            data,
        }
    }
}