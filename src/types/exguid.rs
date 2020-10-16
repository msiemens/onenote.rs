use crate::errors::{ErrorKind, Result};
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

    pub(crate) fn as_option(&self) -> Option<ExGuid> {
        if self.is_nil() {
            None
        } else {
            Some(*self)
        }
    }

    pub(crate) fn from_guid(guid: Guid, value: u32) -> ExGuid {
        ExGuid { guid, value }
    }

    pub(crate) fn parse(reader: Reader) -> Result<ExGuid> {
        let data = reader.get_u8()?;

        if data == 0 {
            return Ok(ExGuid {
                guid: Guid::nil(),
                value: 0,
            });
        }

        if data & 0b111 == 4 {
            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value: (data >> 3) as u32,
            });
        }

        if data & 0b111111 == 32 {
            let value = (reader.get_u8()? as u16) << 2 | (data >> 6) as u16;

            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value: value as u32,
            });
        }

        if data & 0b1111111 == 64 {
            let value = (reader.get_u16()? as u32) << 1 | (data >> 7) as u32;

            return Ok(ExGuid {
                guid: Guid::parse(reader)?,
                value,
            });
        }

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
