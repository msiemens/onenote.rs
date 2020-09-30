use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub enum ParagraphAlignment {
    Unknown,
    Left,
    Center,
    Right,
}

impl ParagraphAlignment {
    pub(crate) fn parse(object: &Object) -> Option<ParagraphAlignment> {
        object
            .props()
            .get(PropertyType::ParagraphAlignment)
            .map(|value| value.to_u8().expect("paragraph alignment is not a u8"))
            .map(|value| match value {
                0 => ParagraphAlignment::Left,
                1 => ParagraphAlignment::Center,
                2 => ParagraphAlignment::Right,
                _ => ParagraphAlignment::Unknown,
            })
    }
}

impl Default for ParagraphAlignment {
    fn default() -> Self {
        ParagraphAlignment::Left
    }
}
