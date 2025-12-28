use crate::Reader;
use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::compact_u64::CompactU64;
use crate::shared::guid::Guid;
use std::fmt;

/// A variable-width encoding of an extended GUID (GUID + 32 bit value)
///
/// See [\[MS-FSSHTTPB\] 2.2.1.7].
///
/// [\[MS-FSSHTTPB\] 2.2.1.7]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/bff58e9f-8222-4fbb-b112-5826d5febedd
#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct ExGuid {
    pub guid: Guid,
    pub value: u32,
}

impl ExGuid {
    pub(crate) fn is_nil(&self) -> bool {
        self.guid.is_nil() && self.value == 0
    }

    pub(crate) fn as_option(&self) -> Option<ExGuid> {
        if self.is_nil() { None } else { Some(*self) }
    }

    pub(crate) fn from_guid(guid: Guid, value: u32) -> ExGuid {
        ExGuid { guid, value }
    }

    pub(crate) fn parse(reader: Reader) -> Result<ExGuid> {
        let data = reader.get_u8()?;

        // A null ExGuid ([FSSHTTPB] 2.2.1.7.1)
        if data == 0 {
            return Ok(ExGuid {
                guid: Guid::nil(),
                value: 0,
            });
        }

        // A ExGuid with a 5 bit value ([FSSHTTPB] 2.2.1.7.2)
        if data & 0b111 == 4 {
            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value: (data >> 3) as u32,
            });
        }

        // A ExGuid with a 10 bit value ([FSSHTTPB] 2.2.1.7.3)
        if data & 0b111111 == 32 {
            let value = (reader.get_u8()? as u16) << 2 | (data >> 6) as u16;

            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value: value as u32,
            });
        }

        // A ExGuid with a 17 bit value ([FSSHTTPB] 2.2.1.7.4)
        if data & 0b1111111 == 64 {
            let value = (reader.get_u16()? as u32) << 1 | (data >> 7) as u32;

            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value,
            });
        }

        // A ExGuid with a 32 bit value ([FSSHTTPB] 2.2.1.7.5)
        if data == 128 {
            let value = reader.get_u32()?;

            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value,
            });
        }

        Err(
            ErrorKind::MalformedData(format!("unexpected ExGuid first byte: {:b}", data).into())
                .into(),
        )
    }

    /// Parse an array of `ExGuid` values.
    ///
    /// See [\[MS-FSSHTTPB\] 2.2.1.8]
    ///
    /// [\[MS-FSSHTTPB\] 2.2.1.8]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/10d6fb35-d630-4ae3-b530-b9e877fc27d3
    pub(crate) fn parse_array(reader: Reader) -> Result<Vec<ExGuid>> {
        let mut values = vec![];

        let count = CompactU64::parse(reader)?.value();
        for _ in 0..count {
            values.push(ExGuid::parse(reader)?);
        }

        Ok(values)
    }
}

impl fmt::Debug for ExGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExGuid {{{}, {}}}", self.guid, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::ExGuid;
    use crate::reader::Reader;
    use crate::shared::guid::Guid;
    use uuid::Uuid;

    fn guid_bytes() -> [u8; 16] {
        [
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF,
        ]
    }

    fn expected_guid() -> Guid {
        Guid(Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").expect("valid test guid"))
    }

    #[test]
    fn test_parse_nil() {
        let mut reader = Reader::new(&[0u8]);
        let guid = ExGuid::parse(&mut reader).unwrap();

        assert!(guid.is_nil());
        assert!(guid.as_option().is_none());
    }

    #[test]
    fn test_parse_5_bit_value() {
        let value = 17u8;
        let mut bytes = vec![(value << 3) | 0b100];
        bytes.extend_from_slice(&guid_bytes());

        let mut reader = Reader::new(&bytes);
        let guid = ExGuid::parse(&mut reader).unwrap();

        assert_eq!(guid.value, value as u32);
        assert_eq!(guid.guid, expected_guid());
    }

    #[test]
    fn test_parse_10_bit_value() {
        let value = 0x155u16;
        let first = 0b100000 | ((value & 0b11) << 6) as u8;
        let second = (value >> 2) as u8;

        let mut bytes = vec![first, second];
        bytes.extend_from_slice(&guid_bytes());

        let mut reader = Reader::new(&bytes);
        let guid = ExGuid::parse(&mut reader).unwrap();

        assert_eq!(guid.value, value as u32);
        assert_eq!(guid.guid, expected_guid());
    }

    #[test]
    fn test_parse_17_bit_value() {
        let value = 0x1ABCDu32;
        let first = 0b1000000 | ((value & 0x1) << 7) as u8;
        let second = ((value >> 1) as u16).to_le_bytes();

        let mut bytes = vec![first, second[0], second[1]];
        bytes.extend_from_slice(&guid_bytes());

        let mut reader = Reader::new(&bytes);
        let guid = ExGuid::parse(&mut reader).unwrap();

        assert_eq!(guid.value, value);
        assert_eq!(guid.guid, expected_guid());
    }

    #[test]
    fn test_parse_32_bit_value() {
        let value = 0xDEAD_BEEFu32;
        let mut bytes = vec![0x80];
        bytes.extend_from_slice(&value.to_le_bytes());
        bytes.extend_from_slice(&guid_bytes());

        let mut reader = Reader::new(&bytes);
        let guid = ExGuid::parse(&mut reader).unwrap();

        assert_eq!(guid.value, value);
        assert_eq!(guid.guid, expected_guid());
    }

    #[test]
    fn test_parse_array() {
        let mut bytes = vec![(2u8 << 1) | 0x1];
        bytes.push(0);

        let value = 3u8;
        bytes.push((value << 3) | 0b100);
        bytes.extend_from_slice(&guid_bytes());

        let mut reader = Reader::new(&bytes);
        let values = ExGuid::parse_array(&mut reader).unwrap();

        assert_eq!(values.len(), 2);
        assert!(values[0].is_nil());
        assert_eq!(values[1].value, value as u32);
    }

    #[test]
    fn test_parse_invalid_first_byte() {
        let mut reader = Reader::new(&[0x02u8]);
        assert!(ExGuid::parse(&mut reader).is_err());
    }
}
