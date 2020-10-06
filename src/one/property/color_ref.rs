use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub enum ColorRef {
    Auto,
    Manual { b: u8, g: u8, r: u8 },
}

impl ColorRef {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<ColorRef> {
        object
            .props()
            .get(prop_type)
            .map(|value| value.to_u32().expect("color ref is not a u32"))
            .map(|value| value.to_le_bytes())
            .map(|value| match value[3] {
                0xFF => ColorRef::Auto,
                0x00 => ColorRef::Manual {
                    r: value[0],
                    g: value[1],
                    b: value[2],
                },
                _ => panic!("invalid color ref: 0x{:08X}", u32::from_le_bytes(value)),
            })
    }
}
