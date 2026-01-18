use crate::Reader;
use crate::errors::{ErrorKind, Result};
use crate::onestore::shared::prop_set::PropertySet;
use std::fmt;

/// A property value.
#[allow(dead_code)]
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

    pub(crate) fn parse(property_id: PropertyId, reader: Reader) -> Result<PropertyValue> {
        let prop_type = property_id.prop_type();

        let value = match prop_type {
            0x1 => PropertyValue::Empty,
            0x2 => PropertyValue::Bool(property_id.bool()),
            0x3 => PropertyValue::U8(reader.get_u8()?),
            0x4 => PropertyValue::U16(reader.get_u16()?),
            0x5 => PropertyValue::U32(reader.get_u32()?),
            0x6 => PropertyValue::U64(reader.get_u64()?),
            0x7 => PropertyValue::parse_vec(reader)?,

            0x8 => PropertyValue::ObjectId,
            0x9 => PropertyValue::ObjectIds(reader.get_u32()?),

            0xA => PropertyValue::ObjectSpaceId,
            0xB => PropertyValue::ObjectSpaceIds(reader.get_u32()?),

            0xC => PropertyValue::ContextId,
            0xD => PropertyValue::ContextIds(reader.get_u32()?),

            0x10 => PropertyValue::parse_property_values(reader)?,
            0x11 => PropertyValue::PropertySet(PropertySet::parse(reader)?),

            v => {
                return Err(ErrorKind::MalformedOneStoreData(
                    format!("unexpected property type: 0x{:x}", v).into(),
                )
                .into());
            }
        };

        Ok(value)
    }

    fn parse_vec(reader: Reader) -> Result<PropertyValue> {
        let size = reader.get_u32()?;
        let data = reader.read(size as usize)?.to_vec();

        Ok(PropertyValue::Vec(data))
    }

    fn parse_property_values(reader: Reader) -> Result<PropertyValue> {
        let size = reader.get_u32()?;

        // Parse property ID

        let id = PropertyId::parse(reader)?;

        // Parse property values

        let values = (0..size)
            .map(|_| PropertySet::parse(reader))
            .collect::<Result<_>>()?;

        Ok(PropertyValue::PropertyValues(id, values))
    }
}

/// A property ID.
///
/// See [\[MS-ONESTORE\] 2.6.6].
///
/// [\[MS-ONESTORE\] 2.6.6]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/17d8c39e-6cc2-4fcd-8d10-aee950fd0ab2
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
        self.0 >> 26 & 0x1f
    }

    pub(crate) fn bool(&self) -> bool {
        self.0 >> 31 == 1
    }

    pub(crate) fn parse(reader: Reader) -> Result<PropertyId> {
        reader.get_u32().map(PropertyId::new)
    }
}

impl fmt::Debug for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PropertyId(0x{:08X})", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::{PropertyId, PropertyValue};
    use crate::reader::Reader;

    fn make_property_id(prop_type: u32, id: u32, bool_flag: bool) -> PropertyId {
        let mut value = (prop_type << 26) | (id & 0x3ffffff);
        if bool_flag {
            value |= 1 << 31;
        }

        PropertyId::new(value)
    }

    #[test]
    fn test_property_bool() {
        assert_eq!(PropertyId::new(0x08001C04).bool(), false);
        assert_eq!(PropertyId::new(0x88001C04).bool(), true);
        assert_eq!(PropertyId::new(0x88001C04).id(), 0x1C04);
        assert_eq!(PropertyId::new(0x88001C04).prop_type(), 0x2);
    }

    #[test]
    fn test_property_value_scalars() {
        let value = PropertyValue::parse(make_property_id(0x1, 0x10, false), &mut Reader::new(&[]))
            .unwrap();
        assert!(matches!(value, PropertyValue::Empty));

        let value =
            PropertyValue::parse(make_property_id(0x2, 0x11, true), &mut Reader::new(&[])).unwrap();
        assert!(matches!(value, PropertyValue::Bool(true)));

        let value = PropertyValue::parse(
            make_property_id(0x3, 0x12, false),
            &mut Reader::new(&[0xAB]),
        )
        .unwrap();
        assert!(matches!(value, PropertyValue::U8(0xAB)));

        let value = PropertyValue::parse(
            make_property_id(0x4, 0x13, false),
            &mut Reader::new(&[0x34, 0x12]),
        )
        .unwrap();
        assert!(matches!(value, PropertyValue::U16(0x1234)));

        let value = PropertyValue::parse(
            make_property_id(0x5, 0x14, false),
            &mut Reader::new(&[0x78, 0x56, 0x34, 0x12]),
        )
        .unwrap();
        assert!(matches!(value, PropertyValue::U32(0x1234_5678)));

        let value = PropertyValue::parse(
            make_property_id(0x6, 0x15, false),
            &mut Reader::new(&[0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01]),
        )
        .unwrap();
        assert!(matches!(value, PropertyValue::U64(0x0123_4567_89AB_CDEF)));
    }

    #[test]
    fn test_property_value_vectors_and_ids() {
        let value = PropertyValue::parse(
            make_property_id(0x7, 0x20, false),
            &mut Reader::new(&[0x03, 0x00, 0x00, 0x00, 0xDE, 0xAD, 0xBE]),
        )
        .unwrap();
        assert!(matches!(value, PropertyValue::Vec(v) if v == vec![0xDE, 0xAD, 0xBE]));

        let value = PropertyValue::parse(make_property_id(0x8, 0x21, false), &mut Reader::new(&[]))
            .unwrap();
        assert!(matches!(value, PropertyValue::ObjectId));

        let value = PropertyValue::parse(
            make_property_id(0x9, 0x22, false),
            &mut Reader::new(&[0xEF, 0xBE, 0xAD, 0xDE]),
        )
        .unwrap();
        assert!(matches!(value, PropertyValue::ObjectIds(0xDEAD_BEEF)));
    }

    #[test]
    fn test_property_value_property_sets() {
        let nested_prop_id = make_property_id(0x5, 0x30, false);
        let nested_prop_id_bytes = nested_prop_id.value().to_le_bytes();

        let property_set_bytes = [
            0x01,
            0x00, // count
            nested_prop_id_bytes[0],
            nested_prop_id_bytes[1],
            nested_prop_id_bytes[2],
            nested_prop_id_bytes[3],
            0x78,
            0x56,
            0x34,
            0x12, // value
        ];

        let value = PropertyValue::parse(
            make_property_id(0x11, 0x31, false),
            &mut Reader::new(&property_set_bytes),
        )
        .unwrap();

        match value {
            PropertyValue::PropertySet(props) => {
                let inner = props.get(nested_prop_id).unwrap();
                assert!(matches!(inner, PropertyValue::U32(0x1234_5678)));
            }
            _ => panic!("expected property set"),
        }

        let value = PropertyValue::parse(
            make_property_id(0x10, 0x32, false),
            &mut Reader::new(&[
                0x01,
                0x00,
                0x00,
                0x00, // size
                nested_prop_id_bytes[0],
                nested_prop_id_bytes[1],
                nested_prop_id_bytes[2],
                nested_prop_id_bytes[3],
                0x01,
                0x00, // count
                nested_prop_id_bytes[0],
                nested_prop_id_bytes[1],
                nested_prop_id_bytes[2],
                nested_prop_id_bytes[3],
                0x78,
                0x56,
                0x34,
                0x12, // value
            ]),
        )
        .unwrap();

        match value {
            PropertyValue::PropertyValues(id, values) => {
                assert_eq!(id, nested_prop_id);
                assert_eq!(values.len(), 1);

                let inner = values[0].get(nested_prop_id).unwrap();
                assert!(matches!(inner, PropertyValue::U32(0x1234_5678)));
            }
            _ => panic!("expected property values"),
        }
    }

    #[test]
    fn test_property_value_invalid_type() {
        let value =
            PropertyValue::parse(make_property_id(0x1F, 0x40, false), &mut Reader::new(&[]));
        assert!(value.is_err());
    }
}
