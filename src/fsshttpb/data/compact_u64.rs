use crate::errors::{ErrorKind, Result};
use crate::Reader;

#[derive(Debug)]
pub(crate) struct CompactU64(u64);

impl CompactU64 {
    pub(crate) fn value(&self) -> u64 {
        self.0
    }

    pub(crate) fn parse(reader: Reader) -> Result<CompactU64> {
        let bytes = reader.bytes();

        let first_byte = bytes.first().copied().ok_or(ErrorKind::UnexpectedEOF)?;

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
                return Err(ErrorKind::UnexpectedEOF.into());
            }

            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);

            reader.advance(3)?;

            return Ok(CompactU64((value >> 3) as u64));
        }

        if first_byte & 8 != 0 {
            if reader.remaining() < 4 {
                return Err(ErrorKind::UnexpectedEOF.into());
            }

            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            reader.advance(4)?;

            return Ok(CompactU64((value >> 4) as u64));
        }

        if first_byte & 16 != 0 {
            if reader.remaining() < 5 {
                return Err(ErrorKind::UnexpectedEOF.into());
            }

            let value =
                u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], 0, 0, 0]);

            reader.advance(5)?;

            return Ok(CompactU64(value >> 5));
        }

        if first_byte & 32 != 0 {
            if reader.remaining() < 6 {
                return Err(ErrorKind::UnexpectedEOF.into());
            }

            let value = u64::from_le_bytes([
                first_byte, bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
            ]);

            reader.advance(6)?;

            return Ok(CompactU64(value >> 6));
        }

        if first_byte & 64 != 0 {
            if reader.remaining() < 7 {
                return Err(ErrorKind::UnexpectedEOF.into());
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

        panic!("unexpected compact u64 type: {:x}", first_byte)
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
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_14_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_21_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0xd4u8, 0x8b, 0x10]))
                .unwrap()
                .value(),
            135546
        );
    }

    #[test]
    fn test_28_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_35_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_42_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_49_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }

    #[test]
    fn test_64_bit() {
        assert_eq!(
            CompactU64::parse(&mut Reader::new(&[0u8])).unwrap().value(),
            0
        );
    }
}
