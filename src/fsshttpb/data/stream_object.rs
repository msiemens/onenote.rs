use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::compact_u64::CompactU64;
use crate::fsshttpb::data::object_types::ObjectType;
use crate::Reader;
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug)]
pub struct ObjectHeader {
    pub compound: bool,
    pub object_type: ObjectType,
    pub length: u64,
}

impl ObjectHeader {
    pub(crate) fn try_parse(reader: Reader, object_type: ObjectType) -> Result<()> {
        Self::try_parse_start(reader, object_type, Self::parse)
    }

    pub(crate) fn parse(reader: Reader) -> Result<ObjectHeader> {
        let header_type = reader.bytes().first().ok_or(ErrorKind::UnexpectedEof)?;

        match header_type & 0b11 {
            0x0 => Self::parse_16(reader),
            0x2 => Self::parse_32(reader),
            _ => Err(ErrorKind::MalformedFssHttpBData(
                format!("unexpected object header type: {:x}", header_type).into(),
            )
            .into()),
        }
    }

    pub(crate) fn try_parse_16(reader: Reader, object_type: ObjectType) -> Result<()> {
        Self::try_parse_start(reader, object_type, Self::parse_16)
    }

    pub(crate) fn parse_16(reader: Reader) -> Result<ObjectHeader> {
        let data = reader.get_u16()?;

        let header_type = data & 0b11;
        if header_type != 0x0 {
            return Err(ErrorKind::MalformedFssHttpBData(
                format!(
                    "unexpected object header type for 16 bit header: 0x{:x}",
                    header_type
                )
                .into(),
            )
            .into());
        }

        let compound = data & 0x4 == 0x4;
        let object_type_value = (data >> 3) & 0x3f;
        let object_type = ObjectType::from_u16(object_type_value).ok_or_else(|| {
            ErrorKind::MalformedFssHttpBData(
                format!("invalid object type: 0x{:x}", object_type_value).into(),
            )
        })?;
        let length = (data >> 9) as u64;

        Ok(ObjectHeader {
            compound,
            object_type,
            length,
        })
    }

    pub(crate) fn try_parse_32(reader: Reader, object_type: ObjectType) -> Result<()> {
        Self::try_parse_start(reader, object_type, Self::parse_32)
    }

    fn parse_32(reader: Reader) -> Result<ObjectHeader> {
        let data = reader.get_u32()?;

        let header_type = data & 0b11;
        if header_type != 0x2 {
            return Err(ErrorKind::MalformedFssHttpBData(
                format!(
                    "unexpected object header type for 32 bit header: 0x{:x}",
                    header_type
                )
                .into(),
            )
            .into());
        }

        let compound = data & 0x4 == 0x4;
        let object_type_value = (data >> 3) & 0x3fff;
        let object_type = ObjectType::from_u32(object_type_value).ok_or_else(|| {
            ErrorKind::MalformedFssHttpBData(
                format!("invalid object type: 0x{:x}", object_type_value).into(),
            )
        })?;
        let mut length = (data >> 17) as u64;

        if length == 0x7fff {
            length = CompactU64::parse(reader)?.value();
        }

        Ok(ObjectHeader {
            compound,
            object_type,
            length,
        })
    }

    pub(crate) fn try_parse_end_16(reader: Reader, object_type: ObjectType) -> Result<()> {
        Self::try_parse_end(reader, object_type, Self::parse_end_16)
    }

    fn parse_end_16(reader: Reader) -> Result<ObjectType> {
        let data = reader.get_u16()?;
        let header_type = data & 0b11;
        if header_type != 0x3 {
            return Err(ErrorKind::MalformedFssHttpBData(
                format!(
                    "unexpected object header type for 16 bit end header: {:x}",
                    header_type
                )
                .into(),
            )
            .into());
        }

        let object_type_value = data >> 2;
        ObjectType::from_u16(object_type_value).ok_or_else(|| {
            ErrorKind::MalformedFssHttpBData(
                format!("invalid object type: 0x{:x}", object_type_value).into(),
            )
            .into()
        })
    }

    pub(crate) fn try_parse_end_8(reader: Reader, object_type: ObjectType) -> Result<()> {
        Self::try_parse_end(reader, object_type, Self::parse_end_8)
    }

    fn parse_end_8(reader: Reader) -> Result<ObjectType> {
        let data = reader.get_u8()?;
        let header_type = data & 0b11;
        if header_type != 0x1 {
            return Err(ErrorKind::MalformedFssHttpBData(
                format!(
                    "unexpected object header type for 8 bit end header: {:x}",
                    header_type
                )
                .into(),
            )
            .into());
        }

        let object_type_value = data >> 2;
        ObjectType::from_u8(object_type_value).ok_or_else(|| {
            ErrorKind::MalformedFssHttpBData(
                format!("invalid object type: 0x{:x}", object_type_value).into(),
            )
            .into()
        })
    }

    pub(crate) fn has_end_8(reader: Reader, object_type: ObjectType) -> Result<bool> {
        let data = reader.bytes().first().ok_or(ErrorKind::UnexpectedEof)?;

        Ok(data & 0b11 == 0x1 && data >> 2 == object_type.to_u8().unwrap())
    }

    fn try_parse_start(
        reader: Reader,
        object_type: ObjectType,
        parse: fn(Reader) -> Result<ObjectHeader>,
    ) -> Result<()> {
        match parse(reader) {
            Ok(header) if header.object_type == object_type => Ok(()),
            Ok(header) => Err(ErrorKind::MalformedFssHttpBData(
                format!("unexpected object type: {:x}", header.object_type).into(),
            )
            .into()),
            Err(e) => Err(e),
        }
    }

    fn try_parse_end(
        reader: Reader,
        object_type: ObjectType,
        parse: fn(Reader) -> Result<ObjectType>,
    ) -> Result<()> {
        match parse(reader) {
            Ok(header) if header == object_type => Ok(()),
            Ok(header) => Err(ErrorKind::MalformedFssHttpBData(
                format!("unexpected object type: {:x}", header).into(),
            )
            .into()),
            Err(e) => Err(e),
        }
    }
}
