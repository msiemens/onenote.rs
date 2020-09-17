use crate::Reader;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub(crate) struct CompactId {
    n: u8,
    guid_index: u32,
}

impl CompactId {
    pub(crate) fn parse(reader: Reader) -> CompactId {
        let data = reader.get_u32_le();

        let n = (data & 0xFF) as u8;
        let guid_index = data >> 8;

        CompactId { n, guid_index }
    }

    pub(crate) fn n(&self) -> u8 {
        self.n
    }

    pub(crate) fn index(&self) -> u32 {
        self.guid_index
    }
}
