use crate::Reader;
use crate::errors::{ErrorKind, Result};

/// A compact unsigned 64-bit integer.
///
/// The first byte encodes the total width of the integer. If the first byte is zero, there is no
/// further data and the integer value is zero. Otherwise the index of the lowest bit with value 1
/// of the first byte indicates the width of the remaining integer data:
/// If the lowest bit is set, the integer data is 1 byte wide; if the second bit is set, the
/// integer data is 2 bytes wide etc.   
///
/// See [\[MS-FSSHTTPB\] 2.2.1.1].
///
/// [\[MS-FSSHTTPB\] 2.2.1.1]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/8eb74ebe-81d1-4569-a29a-308a6128a52f
#[derive(Debug)]
pub(crate) struct CompactU64(u64);

impl CompactU64 {
    pub(crate) fn value(&self) -> u64 {
        self.0
    }

    pub(crate) fn parse(reader: Reader) -> Result<CompactU64> {
        let bytes = reader.bytes();

        let first_byte = bytes.first().copied().ok_or(ErrorKind::UnexpectedEof)?;

        if first_byte == 0 {
            reader.advance(1)?;

            return Ok(CompactU64(0));
        }

        if first_byte & 1 != 0 {
            return Ok(CompactU64((reader.get_u8()? >> 1) as u64));
        }

        if first_byte & 2 != 0 {
            return Ok(CompactU64((reader.get_u16()? >> 2) as u64));
        }

        if first_byte & 4 != 0 {
            if reader.remaining() < 3 {
                return Err(ErrorKind::UnexpectedEof.into());
            }

            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);

            reader.advance(3)?;

            return Ok(CompactU64((value >> 3) as u64));
        }

        if first_byte & 8 != 0 {
            if reader.remaining() < 4 {
                return Err(ErrorKind::UnexpectedEof.into());
            }

            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            reader.advance(4)?;

            return Ok(CompactU64((value >> 4) as u64));
        }

        if first_byte & 16 != 0 {
            if reader.remaining() < 5 {
                return Err(ErrorKind::UnexpectedEof.into());
            }

            let value =
                u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], 0, 0, 0]);

            reader.advance(5)?;

            return Ok(CompactU64(value >> 5));
        }

        if first_byte & 32 != 0 {
            if reader.remaining() < 6 {
                return Err(ErrorKind::UnexpectedEof.into());
            }

            let value = u64::from_le_bytes([
                first_byte, bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
            ]);

            reader.advance(6)?;

            return Ok(CompactU64(value >> 6));
        }

        if first_byte & 64 != 0 {
            if reader.remaining() < 7 {
                return Err(ErrorKind::UnexpectedEof.into());
            }

            let value = u64::from_le_bytes([
                first_byte, bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], 0,
            ]);

            reader.advance(7)?;

            return Ok(CompactU64(value >> 7));
        }

        if first_byte & 128 != 0 {
            reader.advance(1)?;

            return Ok(CompactU64(reader.get_u64()?));
        }

        Err(ErrorKind::MalformedFssHttpBData(
            format!("unexpected compact u64 type: {first_byte:x}").into(),
        )
        .into())
    }
}

#[cfg(test)]
mod test {
    use crate::fsshttpb::data::compact_u64::CompactU64;
    use crate::reader::Reader;

    #[test]
    fn test_zero() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_7_bit() {
        let value = 0x5Au64;
        let encoded = ((value << 1) | 0x1) as u8;
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[encoded]))
                .unwrap()
                .value(),
            value
        );
    }

    #[test]
    fn test_14_bit() {
        let value = 0x1234u64;
        let encoded = (value << 2) | 0x2;
        let bytes = (encoded as u16).to_le_bytes();
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&bytes)).unwrap().value(),
            value
        );
    }

    #[test]
    fn test_21_bit() {
        let value = 0x1ABCDu64;
        let encoded = (value << 3) | 0x4;
        let bytes = encoded.to_le_bytes();
        let input = [bytes[0], bytes[1], bytes[2]];

        assert_eq!(
            CompactU64::parse(&mut Reader::new(&input)).unwrap().value(),
            value
        );
    }

    #[test]
    fn test_28_bit() {
        let value = 0x0ABC_DEF0u64;
        let encoded = (value << 4) | 0x8;
        let bytes = encoded.to_le_bytes();
        let input = [bytes[0], bytes[1], bytes[2], bytes[3]];

        assert_eq!(
            CompactU64::parse(&mut Reader::new(&input)).unwrap().value(),
            value
        );
    }

    #[test]
    fn test_35_bit() {
        let value = 0x1ABCD_EF01u64;
        let encoded = (value << 5) | 0x10;
        let bytes = encoded.to_le_bytes();
        let input = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]];

        assert_eq!(
            CompactU64::parse(&mut Reader::new(&input)).unwrap().value(),
            value
        );
    }

    #[test]
    fn test_42_bit() {
        let value = 0x1234_5678_9Au64;
        let encoded = (value << 6) | 0x20;
        let bytes = encoded.to_le_bytes();
        let input = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]];

        assert_eq!(
            CompactU64::parse(&mut Reader::new(&input)).unwrap().value(),
            value
        );
    }

    #[test]
    fn test_49_bit() {
        let value = 0x1234_5678_9ABu64;
        let encoded = (value << 7) | 0x40;
        let bytes = encoded.to_le_bytes();
        let input = [
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
        ];

        assert_eq!(
            CompactU64::parse(&mut Reader::new(&input)).unwrap().value(),
            value
        );
    }

    #[test]
    fn test_64_bit() {
        let value = 0x0123_4567_89AB_CDEFu64;
        let mut input = Vec::with_capacity(9);
        input.push(0x80);
        input.extend_from_slice(&value.to_le_bytes());

        assert_eq!(
            CompactU64::parse(&mut Reader::new(&input)).unwrap().value(),
            value
        );
    }
}
