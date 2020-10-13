use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    alpha: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn alpha(&self) -> u8 {
        self.alpha
    }

    pub fn r(&self) -> u8 {
        self.r
    }

    pub fn g(&self) -> u8 {
        self.g
    }

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
