use bytes::Buf;

use crate::data::compact_u64::CompactU64;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct ObjectHeader {
    pub(crate) compound: bool,
    pub(crate) object_type: u32,
    pub(crate) length: u64,
}

impl ObjectHeader {
    pub(crate) fn parse(reader: Reader) -> ObjectHeader {
        let header_type = reader.bytes()[0];

        match header_type & 0b11 {
            0x0 => Self::parse_16(reader),
            0x2 => Self::parse_32(reader),
            _ => panic!("unexpected object header type: {:x}", header_type),
        }
    }

    pub(crate) fn parse_16(reader: Reader) -> ObjectHeader {
        let data = reader.get_u16_le();

        let header_type = data & 0b11;
        assert_eq!(header_type, 0x0);

        let compound = data & 0x4 == 0x4;
        let object_type = ((data >> 3) & 0x3f) as u32;
        let length = (data >> 9) as u64;

        ObjectHeader {
            compound,
            object_type,
            length,
        }
    }

    pub(crate) fn parse_32(reader: Reader) -> ObjectHeader {
        let data = reader.get_u32_le();

        let header_type = data & 0b11;
        assert_eq!(header_type, 0x2);

        let compound = data & 0x4 == 0x4;
        let object_type = (data >> 3) & 0x3fff;
        let mut length = (data >> 17) as u64;

        if length == 0x7fff {
            length = CompactU64::parse(reader).value();
        }

        ObjectHeader {
            compound,
            object_type,
            length,
        }
    }

    pub(crate) fn parse_end_16(reader: Reader) -> u32 {
        let data = reader.get_u16_le();
        assert_eq!(data & 0b11, 0x3);

        (data >> 2) as u32
    }

    pub(crate) fn parse_end_8(reader: Reader) -> u32 {
        let data = reader.get_u8();
        assert_eq!(data & 0b11, 0x1);

        (data >> 2) as u32
    }

    pub(crate) fn try_parse_end_8(reader: Reader, object_type: u8) -> Option<u32> {
        let data = reader.bytes()[0];

        if data & 0b11 == 0x1 && data >> 2 == object_type {
            Some(ObjectHeader::parse_end_8(reader))
        } else {
            None
        }
    }
}
