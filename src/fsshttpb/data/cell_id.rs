use crate::Reader;
use crate::errors::Result;
use crate::fsshttpb::data::compact_u64::CompactU64;
use crate::fsshttpb::data::exguid::ExGuid;

/// A FSSHTTP cell identifier.
///
/// See [\[MS-FSSHTTPB\] 2.2.1.10] and [\[MS-FSSHTTPB\] 2.2.1.11].
///
/// [\[MS-FSSHTTPB\] 2.2.1.10]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/75bf8297-ef9c-458a-95a3-ad6265bfa864
/// [\[MS-FSSHTTPB\] 2.2.1.11]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/d3f4d22d-6fb4-4032-8587-f3eb9c256e45
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellId(pub ExGuid, pub ExGuid);

impl CellId {
    pub(crate) fn parse(reader: Reader) -> Result<CellId> {
        let first = ExGuid::parse(reader)?;
        let second = ExGuid::parse(reader)?;

        Ok(CellId(first, second))
    }

    pub(crate) fn parse_array(reader: Reader) -> Result<Vec<CellId>> {
        let mut values = vec![];

        let count = CompactU64::parse(reader)?.value();
        for _ in 0..count {
            values.push(CellId::parse(reader)?);
        }

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::CellId;
    use crate::fsshttpb::data::exguid::ExGuid;
    use crate::reader::Reader;
    use crate::shared::guid::Guid;
    use uuid::Uuid;

    fn guid_bytes() -> [u8; 16] {
        [
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF,
        ]
    }

    fn guid() -> Guid {
        Guid(Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").expect("valid test guid"))
    }

    fn encode_exguid_5bit(value: u8) -> Vec<u8> {
        let mut bytes = vec![(value << 3) | 0b100];
        bytes.extend_from_slice(&guid_bytes());
        bytes
    }

    #[test]
    fn test_parse_cell_id() {
        let mut bytes = vec![0];
        bytes.extend_from_slice(&encode_exguid_5bit(7));

        let mut reader = Reader::new(&bytes);
        let cell = CellId::parse(&mut reader).unwrap();

        assert!(cell.0.is_nil());
        assert_eq!(cell.1, ExGuid::from_guid(guid(), 7));
    }

    #[test]
    fn test_parse_cell_id_array() {
        let mut bytes = vec![(2u8 << 1) | 0x1];

        let mut first = vec![0];
        first.extend_from_slice(&encode_exguid_5bit(1));

        let mut second = vec![0];
        second.extend_from_slice(&encode_exguid_5bit(2));

        bytes.extend_from_slice(&first);
        bytes.extend_from_slice(&second);

        let mut reader = Reader::new(&bytes);
        let values = CellId::parse_array(&mut reader).unwrap();

        assert_eq!(values.len(), 2);
        assert!(values[0].0.is_nil());
        assert_eq!(values[0].1.value, 1);
        assert_eq!(values[1].1.value, 2);
    }
}
