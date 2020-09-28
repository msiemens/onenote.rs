use crate::types::compact_u64::CompactU64;
use crate::types::object_types::ObjectType;
use crate::Reader;
use bytes::Buf;
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug)]
pub struct ObjectHeader {
    pub compound: bool,
    pub object_type: ObjectType,
    pub length: u64,
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
        let object_type_value = (data >> 3) & 0x3f;
        let object_type = ObjectType::from_u16(object_type_value)
            .unwrap_or_else(|| panic!("invalid object type: 0x{:x}", object_type_value));
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
        let object_type_value = (data >> 3) & 0x3fff;
        let object_type = ObjectType::from_u32(object_type_value)
            .unwrap_or_else(|| panic!("invalid object type: 0x{:x}", object_type_value));
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

    pub(crate) fn parse_end_16(reader: Reader) -> ObjectType {
        let data = reader.get_u16_le();
        assert_eq!(data & 0b11, 0x3);

        let object_type_value = data >> 2;
        ObjectType::from_u16(object_type_value)
            .unwrap_or_else(|| panic!("invalid object type: 0x{:x}", object_type_value))
    }

    pub(crate) fn parse_end_8(reader: Reader) -> ObjectType {
        let data = reader.get_u8();
        assert_eq!(data & 0b11, 0x1);

        let object_type_value = data >> 2;
        ObjectType::from_u8(object_type_value)
            .unwrap_or_else(|| panic!("invalid object type: 0x{:x}", object_type_value))
    }

    // FIXME: Turn this into a `has_end_8()`
    pub(crate) fn try_parse_end_8(reader: Reader, object_type: ObjectType) -> Option<ObjectType> {
        let data = reader.bytes()[0];

        if data & 0b11 == 0x1 && data >> 2 == object_type.to_u8().unwrap() {
            Some(ObjectHeader::parse_end_8(reader))
        } else {
            None
        }
    }
}
