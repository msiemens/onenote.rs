use uuid::Uuid;

use crate::Reader;

#[derive(Debug, PartialEq)]
pub(crate) struct Guid(Uuid);

impl Guid {
    pub(crate) fn parse(reader: Reader) -> Guid {
        let v = reader.get_u128();

        Guid(Uuid::from_bytes([
            // Little Endian
            (v >> 96) as u8,
            (v >> 104) as u8,
            (v >> 112) as u8,
            (v >> 120) as u8,
            // Little Endian
            (v >> 80) as u8,
            (v >> 88) as u8,
            // Little endian
            (v >> 64) as u8,
            (v >> 72) as u8,
            // Big Endian
            (v >> 56) as u8,
            (v >> 48) as u8,
            (v >> 40) as u8,
            (v >> 32) as u8,
            (v >> 24) as u8,
            (v >> 16) as u8,
            (v >> 8) as u8,
            v as u8,
        ]))
    }

    pub(crate) fn nil() -> Guid {
        Guid(Uuid::nil())
    }
}
