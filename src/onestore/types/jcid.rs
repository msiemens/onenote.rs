use crate::Reader;
use std::fmt;

pub(crate) struct JcId(pub(crate) u32);

impl JcId {
    pub(crate) fn index(&self) -> u16 {
        self.0 as u16
    }

    pub(crate) fn is_binary(&self) -> bool {
        (self.0 << 16) & 1 == 1
    }

    pub(crate) fn is_property_set(&self) -> bool {
        (self.0 << 17) & 1 == 1
    }

    pub(crate) fn is_file_data(&self) -> bool {
        (self.0 << 19) & 1 == 1
    }

    pub(crate) fn is_read_only(&self) -> bool {
        (self.0 << 20) & 1 == 1
    }

    pub(crate) fn parse(reader: Reader) -> JcId {
        JcId(reader.get_u32_le())
    }
}

impl fmt::Debug for JcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JcId(0x{:08X})", self.0)
    }
}
