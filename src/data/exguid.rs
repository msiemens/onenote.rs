use crate::data::compact_u64::CompactU64;
use crate::data::guid::Guid;
use crate::Reader;

#[derive(Debug)]
pub(crate) struct ExGuid {
    uuid: Guid,
    value: u32,
}

impl ExGuid {
    pub(crate) fn parse(reader: Reader) -> ExGuid {
        let data = reader.get_u8();

        if data == 0 {
            return ExGuid {
                uuid: Guid::nil(),
                value: 0,
            };
        }

        if data & 0b111 == 4 {
            return ExGuid {
                uuid: Guid::parse(reader),
                value: (data >> 3) as u32,
            };
        }

        if data & 0b111111 == 32 {
            let value = (data as u16 & 0b11000000) << 8 | reader.get_u8() as u16;

            return ExGuid {
                uuid: Guid::parse(reader),
                value: value as u32,
            };
        }

        if data & 0b1111111 == 64 {
            let value = (data as u32 & 0b10000000) << 16 | reader.get_u16_le() as u32;

            return ExGuid {
                uuid: Guid::parse(reader),
                value,
            };
        }

        if data == 128 {
            let value = reader.get_u32_le();

            return ExGuid {
                uuid: Guid::parse(reader),
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
}
