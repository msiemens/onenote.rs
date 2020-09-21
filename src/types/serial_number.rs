use crate::types::guid::Guid;
use crate::Reader;

#[derive(Debug)]
pub struct SerialNumber {
    pub guid: Guid,
    pub serial: u64,
}

impl SerialNumber {
    pub(crate) fn parse(reader: Reader) -> SerialNumber {
        let serial_type = reader.get_u8();

        if serial_type == 0 {
            return SerialNumber {
                guid: Guid::nil(),
                serial: 0,
            };
        }

        let guid = Guid::parse(reader);
        let serial = reader.get_u64_le();

        SerialNumber { guid, serial }
    }
}