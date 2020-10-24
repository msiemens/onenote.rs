use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

/// An RGB color value.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ColorRef {
    /// Determined by the application.
    Auto,

    /// A manually specified color
    Manual {
        /// The color's red value.
        r: u8,
        /// The color's green value.
        g: u8,
        /// The color's blue value
        b: u8,
    },
}

impl ColorRef {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<ColorRef>> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value.to_u32().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("color ref is not a u32".into())
                })
            })
            .transpose()?
            .map(|value| value.to_le_bytes())
            .map(|value| match value[3] {
                0xFF => Ok(ColorRef::Auto),
                0x00 => Ok(ColorRef::Manual {
                    r: value[0],
                    g: value[1],
                    b: value[2],
                }),
                _ => Err(ErrorKind::MalformedOneNoteFileData(
                    format!("invalid color ref: 0x{:08X}", u32::from_le_bytes(value)).into(),
                )
                .into()),
            })
            .transpose()
    }
}
