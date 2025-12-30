use crate::errors::Result;
use crate::one::property::{PropertyType, simple};
use crate::onestore::object::Object;
use widestring::U16String;

/// A math inline object.
#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) object_type: Option<u32>,
    pub(crate) arg_count: Option<u32>,
    pub(crate) column: Option<u8>,
    pub(crate) align: Option<u8>,
    pub(crate) char: Option<char>,
    pub(crate) char1: Option<char>,
    pub(crate) char2: Option<char>,
}

impl Data {
    pub(crate) fn parse(object: &Object) -> Result<Data> {
        let object_type = simple::parse_u32(PropertyType::MathInlineObjectType, object)?;
        let arg_count = simple::parse_u32(PropertyType::MathInlineObjectCount, object)?;
        let column = simple::parse_u8(PropertyType::MathInlineObjectCol, object)?;
        let align = simple::parse_u8(PropertyType::MathInlineObjectAlign, object)?;
        let char = simple::parse_u16(PropertyType::MathInlineObjectChar, object)?
            .map(parse_utf16_char)
            .transpose()?
            .flatten();
        let char1 = simple::parse_u16(PropertyType::MathInlineObjectChar1, object)?
            .map(parse_utf16_char)
            .transpose()?
            .flatten();
        let char2 = simple::parse_u16(PropertyType::MathInlineObjectChar2, object)?
            .map(parse_utf16_char)
            .transpose()?
            .flatten();

        Ok(Data {
            object_type,
            arg_count,
            column,
            align,
            char,
            char1,
            char2,
        })
    }
}

fn parse_utf16_char(c: u16) -> Result<Option<char>> {
    Ok(U16String::from_vec([c]).to_string()?.chars().next())
}
