use crate::errors::Result;
use crate::types::compact_u64::CompactU64;
use crate::types::guid::Guid;
use crate::Reader;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct ExGuid {
    pub guid: Guid,
    pub value: u32,
}

impl ExGuid {
    pub(crate) fn is_nil(&self) -> bool {
        self.guid.is_nil() && self.value == 0
    }

    pub(crate) fn from_guid(guid: Guid, value: u32) -> ExGuid {
        ExGuid { guid, value }
    }

    pub(crate) fn parse(reader: Reader) -> ExGuid {
        let data = reader.get_u8();

        if data == 0 {
            return ExGuid {
                guid: Guid::nil(),
                value: 0,
            };
        }

        if data & 0b111 == 4 {
            return ExGuid {
                guid: Guid::parse(reader),
                value: (data >> 3) as u32,
            };
        }

        if data & 0b111111 == 32 {
            let value = (reader.get_u8() as u16) << 2 | (data >> 6) as u16;

            return ExGuid {
                guid: Guid::parse(reader),
                value: value as u32,
            };
        }

        if data & 0b1111111 == 64 {
            let value = (reader.get_u16() as u32) << 1 | (data >> 7) as u32;

            return ExGuid {
                guid: Guid::parse(reader),
                value,
            };
        }

        if data == 128 {
            let value = reader.get_u32_le();

            return ExGuid {
                guid: Guid::parse(reader),
                value,
            };
        }

        panic!("unexpected ExGuid first byte: {:b}", data)
    }

    pub(crate) fn parse_array(reader: Reader) -> Vec<ExGuid> {
        let mut values = vec![];

        let count = CompactU64::parse(reader).value();
        for _ in 0..count {
            values.push(ExGuid::parse(reader));
        }

        values
    }

    pub(crate) fn parse_str(guid: &str, n: u32) -> Result<ExGuid> {
        Ok(ExGuid {
            guid: Guid::from_str(guid)?,
            value: n,
        })
    }
}

impl fmt::Debug for ExGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExGuid {{{}, {}}}", self.guid, self.value)
    }
}