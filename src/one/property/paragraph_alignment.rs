use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParagraphAlignment {
    Unknown,
    Left,
    Center,
    Right,
}

impl ParagraphAlignment {
    pub(crate) fn parse(object: &Object) -> Result<Option<ParagraphAlignment>> {
        let alignment = object
            .props()
            .get(PropertyType::ParagraphAlignment)
            .map(|value| {
                value.to_u8().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("paragraph alignment is not a u8".into())
                })
            })
            .transpose()?
            .map(|value| match value {
                0 => ParagraphAlignment::Left,
                1 => ParagraphAlignment::Center,
                2 => ParagraphAlignment::Right,
                _ => ParagraphAlignment::Unknown,
            });

        Ok(alignment)
    }
}

impl Default for ParagraphAlignment {
    fn default() -> Self {
        ParagraphAlignment::Left
    }
}
