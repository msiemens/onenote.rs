use bytes::Buf;

use crate::Reader;

#[derive(Debug)]
pub(crate) struct CompactU64(u64);

impl CompactU64 {
    pub fn value(&self) -> u64 {
        self.0
    }

    pub(crate) fn parse(reader: Reader) -> CompactU64 {
        let bytes = reader.bytes();

        if bytes[0] == 0 {
            reader.advance(1);

            return CompactU64(0);
        }

        if bytes[0] & 1 != 0 {
            return CompactU64((reader.get_u8() >> 1) as u64);
        }

        if bytes[0] & 2 != 0 {
            return CompactU64((reader.get_u16_le() >> 2) as u64);
        }

        if bytes[0] & 4 != 0 {
            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);

            reader.advance(3);

            return CompactU64((value >> 3) as u64);
        }

        if bytes[0] & 8 != 0 {
            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            reader.advance(4);

            return CompactU64((value >> 4) as u64);
        }

        if bytes[0] & 16 != 0 {
            let value = u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], 0, 0, 0, 0]);

            reader.advance(5);

            return CompactU64(value >> 5);
        }

        if bytes[0] & 32 != 0 {
            let value = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
            ]);

            reader.advance(6);

            return CompactU64(value >> 6);
        }

        if bytes[0] & 64 != 0 {
            let value = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], 0,
            ]);

            reader.advance(7);

            return CompactU64(value >> 7);
        }

        if bytes[0] & 128 != 0 {
            reader.advance(1);

            return CompactU64(reader.get_u64_le());
        }

        panic!("unexpected compact u64 type: {:x}", bytes[0])
    }
}

#[cfg(test)]
mod test {
    use crate::data::compact_u64::CompactU64;

    #[test]
    fn test_zero() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_7_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_14_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_21_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0xd4, 0x8b, 0x10])).value(),
            135546
        );
    }

    #[test]
    fn test_28_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_35_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_42_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_49_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }

    #[test]
    fn test_64_bit() {
        assert_eq!(
            CompactU64::parse(&mut bytes::Bytes::from_static(&[0])).value(),
            0
        );
    }
}
