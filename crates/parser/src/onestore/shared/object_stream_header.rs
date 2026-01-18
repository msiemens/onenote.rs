use crate::Reader;
use crate::errors::Result;

/// An object stream header.
///
/// See [\[MS-ONESTORE\] 2.6.5].
///
/// [\[MS-ONESTORE\] 2.6.5]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/34497a17-3623-4e1d-9488-a2e111a9a279
#[derive(Debug)]
pub(crate) struct ObjectStreamHeader {
    pub(crate) count: u32,
    pub(crate) extended_streams_present: bool,
    pub(crate) osid_stream_not_present: bool,
}

impl ObjectStreamHeader {
    pub(crate) fn parse(reader: Reader) -> Result<ObjectStreamHeader> {
        let data = reader.get_u32()?;

        let count = data & 0xFFFFFF;
        let extended_streams_present = (data >> 30) & 0x1 != 0;
        let osid_stream_not_present = (data >> 31) != 0;

        Ok(ObjectStreamHeader {
            count,
            extended_streams_present,
            osid_stream_not_present,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ObjectStreamHeader;
    use crate::reader::Reader;

    #[test]
    fn test_parse_header_flags() {
        let count: u32 = 0x12_3456;
        let data: u32 = count | (1 << 30) | (1 << 31);
        let bytes = data.to_le_bytes();

        let header = ObjectStreamHeader::parse(&mut Reader::new(&bytes)).unwrap();

        assert_eq!(header.count, count);
        assert!(header.extended_streams_present);
        assert!(header.osid_stream_not_present);
    }
}
