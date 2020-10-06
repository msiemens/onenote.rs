use crate::one::property::color_ref::ColorRef;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) last_modified: Time,
    pub(crate) list_font: Option<String>,
    pub(crate) list_restart: Option<i32>,
    pub(crate) list_format: Vec<char>,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) language_code: Option<u32>,
    pub(crate) font: Option<String>,
    pub(crate) font_size: Option<u16>,
    pub(crate) font_color: Option<ColorRef>,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::NumberListNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("number list has no last modified time");
    let list_font = simple::parse_string(PropertyType::ListFont, object);
    let list_restart =
        simple::parse_u32(PropertyType::ListRestart, object).map(|value| value as i32);
    let list_format = simple::parse_vec_u16(PropertyType::NumberListFormat, object)
        .map(parse_list_format)
        .expect("number list has no list format");
    let bold = simple::parse_bool(PropertyType::Bold, object).unwrap_or_default();
    let italic = simple::parse_bool(PropertyType::Italic, object).unwrap_or_default();
    let language_code = simple::parse_u32(PropertyType::LanguageID, object);
    let font = simple::parse_string(PropertyType::Font, object);
    let font_size = simple::parse_u16(PropertyType::FontSize, object);
    let font_color = ColorRef::parse(PropertyType::FontColor, object);

    Data {
        last_modified,
        list_font,
        list_restart,
        list_format,
        bold,
        italic,
        language_code,
        font,
        font_size,
        font_color,
    }
}

fn parse_list_format(data: Vec<u16>) -> Vec<char> {
    decode_utf16(data[1..].iter().copied())
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect()
}
