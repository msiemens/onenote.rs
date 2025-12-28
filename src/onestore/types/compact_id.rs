use crate::Reader;
use crate::errors::Result;

/// A compact reference to an ID in the mapping table.
///
/// See [\[MS-ONESTORE\] 2.2.2].
///
/// [\[MS-ONESTORE\] 2.2.2]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/8de02f24-9b9c-48a9-bd26-5c0230814bf4
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub(crate) struct CompactId {
    n: u8,
    guid_index: u32,
}

impl CompactId {
    pub(crate) fn parse(reader: Reader) -> Result<CompactId> {
        let data = reader.get_u32()?;

        let n = (data & 0xFF) as u8;
        let guid_index = data >> 8;

        Ok(CompactId { n, guid_index })
    }
}

#[cfg(test)]
mod tests {
    use super::CompactId;
    use crate::reader::Reader;

    #[test]
    fn test_parse_compact_id() {
        let data = [0xDD, 0xCC, 0xBB, 0xAA];
        let compact = CompactId::parse(&mut Reader::new(&data)).unwrap();

        assert_eq!(compact.n, 0xDD);
        assert_eq!(compact.guid_index, 0xAABB_CC);
    }
}
