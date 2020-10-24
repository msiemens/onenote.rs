use crate::errors::Result;
use crate::fsshttpb::data::compact_u64::CompactU64;
use crate::Reader;

pub(crate) struct BinaryItem(Vec<u8>);

impl BinaryItem {
    pub(crate) fn parse(reader: Reader) -> Result<BinaryItem> {
        let size = CompactU64::parse(reader)?.value();
        let data = reader.read(size as usize)?.to_vec();

        Ok(BinaryItem(data))
    }

    pub(crate) fn value(self) -> Vec<u8> {
        self.0
    }
}
