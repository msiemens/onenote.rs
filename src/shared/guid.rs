use crate::Reader;
use crate::errors::Result;
use std::fmt;
use uuid::Uuid;

/// A global UUID.
///
/// Microsoft is using weird mixed endianness in their GUIDs which we have to consider when
/// parsing a GUID from a stream of bytes.
///
/// See also [\[1\]] and [\[2\]].
///
/// [\[1\]]: https://stackoverflow.com/questions/10190817/guid-byte-order-in-net
/// [\[2\]]: https://docs.microsoft.com/en-us/dotnet/api/system.guid.tobytearray?view=net-5.0#remarks
#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Guid(pub Uuid);

impl Guid {
    pub(crate) fn from_str(value: &str) -> Result<Guid> {
        Uuid::parse_str(value).map(Guid).map_err(|e| e.into())
    }

    pub(crate) fn parse(reader: Reader) -> Result<Guid> {
        // Read as little endian
        let v = reader.get_u128()?;

        let guid = Guid(Uuid::from_bytes([
            // Big Endian
            (v >> 24) as u8,
            (v >> 16) as u8,
            (v >> 8) as u8,
            v as u8,
            // Big Endian
            (v >> 40) as u8,
            (v >> 32) as u8,
            // Big Endian
            (v >> 56) as u8,
            (v >> 48) as u8,
            // Little Endian
            (v >> 64) as u8,
            (v >> 72) as u8,
            (v >> 80) as u8,
            (v >> 88) as u8,
            (v >> 96) as u8,
            (v >> 104) as u8,
            (v >> 112) as u8,
            (v >> 120) as u8,
        ]));

        Ok(guid)
    }

    pub(crate) fn nil() -> Guid {
        Guid(Uuid::nil())
    }

    pub(crate) fn is_nil(&self) -> bool {
        self.0.is_nil()
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{:X}}}", self.0)
    }
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Guid {}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::Guid;
    use crate::reader::Reader;
    use uuid::Uuid;

    #[test]
    fn test_parse_mixed_endian() {
        let bytes = [
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF,
        ];

        let guid = Guid::parse(&mut Reader::new(&bytes)).unwrap();
        let expected = Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").unwrap();

        assert_eq!(guid.0, expected);
        assert_eq!(
            format!("{}", guid),
            "{00112233-4455-6677-8899-AABBCCDDEEFF}"
        );
    }

    #[test]
    fn test_nil_guid() {
        let guid = Guid::nil();
        assert!(guid.is_nil());
        assert_eq!(guid.0, Uuid::nil());
    }
}
