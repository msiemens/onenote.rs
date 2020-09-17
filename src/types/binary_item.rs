use crate::types::compact_u64::CompactU64;
use crate::Reader;

pub(crate) struct BinaryItem(Vec<u8>);

impl BinaryItem {
    pub(crate) fn parse(reader: Reader) -> BinaryItem {
        let size = CompactU64::parse(reader).value();
        let data = reader.bytes()[0..(size as usize)].to_vec();
        reader.advance(size as usize);

        BinaryItem(data)
    }

    pub(crate) fn value(self) -> Vec<u8> {
        self.0
    }
}
