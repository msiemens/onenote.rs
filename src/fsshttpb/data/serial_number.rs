use crate::Reader;
use crate::errors::Result;
use crate::shared::guid::Guid;

/// A variable-width serial number.
///
/// See [\[MS-FSSHTTPB\] 2.2.1.9].
///
/// [\[MS-FSSHTTPB\] 2.2.1.9]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/9db15fa4-0dc2-4b17-b091-d33886d8a0f6
#[derive(Debug)]
#[allow(dead_code)]
pub struct SerialNumber {
    pub guid: Guid,
    pub serial: u64,
}

impl SerialNumber {
    pub(crate) fn parse(reader: Reader) -> Result<SerialNumber> {
        let serial_type = reader.get_u8()?;

        // A null-value ([FSSHTTPB] 2.2.1.9.1)
        if serial_type == 0 {
            return Ok(SerialNumber {
                guid: Guid::nil(),
                serial: 0,
            });
        }

        // A serial number with a 64 bit value ([FSSHTTPB] 2.2.1.9.2)
        let guid = Guid::parse(reader)?;
        let serial = reader.get_u64()?;

        Ok(SerialNumber { guid, serial })
    }
}

#[cfg(test)]
mod tests {
    use super::SerialNumber;
    use crate::reader::Reader;
    use crate::shared::guid::Guid;
    use uuid::Uuid;

    fn guid_bytes() -> [u8; 16] {
        [
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF,
        ]
    }

    fn guid() -> Guid {
        Guid(Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").expect("valid test guid"))
    }

    #[test]
    fn test_parse_nil_serial_number() {
        let mut reader = Reader::new(&[0u8]);
        let serial = SerialNumber::parse(&mut reader).unwrap();

        assert!(serial.guid.is_nil());
        assert_eq!(serial.serial, 0);
    }

    #[test]
    fn test_parse_serial_number() {
        let serial_value = 0x0123_4567_89AB_CDEFu64;
        let mut bytes = vec![1u8];
        bytes.extend_from_slice(&guid_bytes());
        bytes.extend_from_slice(&serial_value.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        let serial = SerialNumber::parse(&mut reader).unwrap();

        assert_eq!(serial.guid, guid());
        assert_eq!(serial.serial, serial_value);
    }
}
