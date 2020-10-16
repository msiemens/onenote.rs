use crate::errors::{ErrorKind, Result};
use crate::one::property::color_ref::ColorRef;
use crate::one::property_set::number_list_node;
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct List {
    pub(crate) list_font: Option<String>,
    pub(crate) list_restart: Option<i32>,
    pub(crate) list_format: Vec<char>,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    // pub(crate) language_code: Option<u32>,
    pub(crate) font: Option<String>,
    pub(crate) font_size: Option<u16>,
    pub(crate) font_color: Option<ColorRef>,
}

impl List {
    pub fn list_font(&self) -> Option<&str> {
        self.list_font.as_deref()
    }

    pub fn list_restart(&self) -> Option<i32> {
        self.list_restart
    }

    pub fn list_format(&self) -> &[char] {
        &self.list_format
    }

    pub fn bold(&self) -> bool {
        self.bold
    }

    pub fn italic(&self) -> bool {
        self.italic
    }

    pub fn font(&self) -> Option<&str> {
        self.font.as_deref()
    }

    pub fn font_size(&self) -> Option<u16> {
        self.font_size
    }

    pub fn font_color(&self) -> Option<ColorRef> {
        self.font_color
    }
}

pub(crate) fn parse_list(list_id: ExGuid, space: &ObjectSpace) -> Result<List> {
    let object = space
        .get_object(list_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("rich text content is missing".into()))?;
    let data = number_list_node::parse(object)?;

    // TODO: Parse language code

    let list = List {
        list_font: data.list_font,
        list_restart: data.list_restart,
        list_format: data.list_format,
        bold: data.bold,
        italic: data.italic,
        font: data.font,
        font_size: data.font_size,
        font_color: data.font_color,
    };

    Ok(list)
}
