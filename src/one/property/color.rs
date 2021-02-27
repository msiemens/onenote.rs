use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

/// A RGBA color value.
///
/// See [\[MS-ONE\] 2.2.7]
///
/// [\[MS-ONE\] 2.2.7]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/6e4a87f9-18f0-4ad6-bc7d-0f326d61e136
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    alpha: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    /// The color's transparency value.
    pub fn alpha(&self) -> u8 {
        self.alpha
    }

    /// The color's red value.
    pub fn r(&self) -> u8 {
        self.r
    }

    /// The color's green value.
    pub fn g(&self) -> u8 {
        self.g
    }

    /// The color's blue value.
    pub fn b(&self) -> u8 {
        self.b
    }
}

impl Color {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<Color>> {
        let color = object
            .props()
            .get(prop_type)
            .map(|value| {
                value
                    .to_u32()
                    .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("color is not a u32".into()))
            })
            .transpose()?
            .map(|value| value.to_le_bytes())
            .map(|value| Color {
                alpha: 255 - value[3],
                r: value[0],
                g: value[1],
                b: value[2],
            });

        Ok(color)
    }
}
