use crate::onestore::types::prop_set::PropertySet;
use crate::Reader;
use bytes::Buf;
use std::fmt;

#[derive(Debug, Clone)]
pub(crate) enum PropertyValue {
    Empty,
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Vec(Vec<u8>),
    ObjectId,
    ObjectIds(u32),
    ObjectSpaceId,
    ObjectSpaceIds(u32),
    ContextId,
    ContextIds(u32),
    PropertyValues(PropertyId, Vec<PropertySet>),
    PropertySet(PropertySet),
}

impl PropertyValue {
    pub(crate) fn to_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_u8(&self) -> Option<u8> {
        if let Self::U8(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_u16(&self) -> Option<u16> {
        if let Self::U16(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_u32(&self) -> Option<u32> {
        if let Self::U32(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_u64(&self) -> Option<u64> {
        if let Self::U64(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_vec(&self) -> Option<&[u8]> {
        if let Self::Vec(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn to_object_id(&self) -> Option<()> {
        if let Self::ObjectId = self {
            Some(())
        } else {
            None
        }
    }

    pub(crate) fn to_object_ids(&self) -> Option<u32> {
        if let Self::ObjectIds(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_object_space_ids(&self) -> Option<u32> {
        if let Self::ObjectSpaceIds(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub(crate) fn to_property_values(&self) -> Option<(PropertyId, &[PropertySet])> {
        if let Self::PropertyValues(id, values) = self {
            Some((*id, values))
        } else {
            None
        }
    }
    //
    // pub(crate) fn to_property_set(&self) -> Option<&PropertySet> {
    //     if let Self::PropertySet(props) = self {
    //         Some(props)
    //     } else {
    //         None
    //     }
    // }

    pub(crate) fn parse(property_id: PropertyId, reader: Reader) -> PropertyValue {
        let prop_type = property_id.prop_type();

        match prop_type {
            0x1 => PropertyValue::Empty,
            0x2 => PropertyValue::Bool(property_id.bool()),
            0x3 => PropertyValue::U8(reader.get_u8()),
            0x4 => PropertyValue::U16(reader.get_u16_le()),
            0x5 => PropertyValue::U32(reader.get_u32_le()),
            0x6 => PropertyValue::U64(reader.get_u64_le()),
            0x7 => PropertyValue::parse_vec(reader),

            0x8 => PropertyValue::ObjectId,
            0x9 => PropertyValue::ObjectIds(reader.get_u32_le()),

            0xA => PropertyValue::ObjectSpaceId,
            0xB => PropertyValue::ObjectSpaceIds(reader.get_u32_le()),

            0xC => PropertyValue::ContextId,
            0xD => PropertyValue::ContextIds(reader.get_u32_le()),

            0x10 => PropertyValue::parse_property_values(reader),
            0x11 => PropertyValue::PropertySet(PropertySet::parse(reader)),
            v => panic!("unexpected property type: 0x{:x}", v),
        }
    }

    fn parse_vec(reader: Reader) -> PropertyValue {
        let size = reader.get_u32_le();
        let data = reader.bytes()[0..(size as usize)].to_vec();
        reader.advance(size as usize);

        PropertyValue::Vec(data)
    }

    fn parse_property_values(reader: Reader) -> PropertyValue {
        let size = reader.get_u32_le();

        // Parse property ID

        let id = PropertyId::parse(reader);

        // Parse property values

        let values = (0..size).map(|_| PropertySet::parse(reader)).collect();

        PropertyValue::PropertyValues(id, values)
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct PropertyId(u32);

impl PropertyId {
    pub(crate) fn new(value: u32) -> PropertyId {
        PropertyId(value)
    }

    pub(crate) fn value(&self) -> u32 {
        self.0
    }

    pub(crate) fn id(&self) -> u32 {
        self.0 & 0x3ffffff
    }

    pub(crate) fn prop_type(&self) -> u32 {
        self.0 >> 26 & 0b011111
    }

    pub(crate) fn bool(&self) -> bool {
        self.0 >> 31 == 1
    }

    pub(crate) fn parse(reader: Reader) -> PropertyId {
        PropertyId(reader.get_u32_le())
    }
}

impl fmt::Debug for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PropertyId(0x{:08X})", self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::onestore::types::property::PropertyId;

    #[test]
    fn test_property_bool() {
        assert_eq!(PropertyId::new(0x08001C04).bool(), false);
        assert_eq!(PropertyId::new(0x88001C04).bool(), true);
        assert_eq!(PropertyId::new(0x88001C04).id(), 0x1C04);
        assert_eq!(PropertyId::new(0x88001C04).prop_type(), 0x2);
    }
}
