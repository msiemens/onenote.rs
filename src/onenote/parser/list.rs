use crate::one::property::color_ref::ColorRef;
use crate::one::property_set::number_list_node;
use crate::onestore::revision::Revision;
use crate::onestore::OneStore;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
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

pub(crate) fn parse_list(list_id: ExGuid, rev: &Revision, store: &OneStore) -> List {
    let object = rev
        .resolve_object(list_id, store)
        .expect("rich text content is missing");
    let data = number_list_node::parse(object);

    // TODO: Parse language code

    List {
        list_font: data.list_font().map(String::from),
        list_restart: data.list_restart(),
        list_format: data.list_format().to_vec(),
        bold: data.bold(),
        italic: data.italic(),
        font: data.font().map(String::from),
        font_size: data.font_size(),
        font_color: data.font_color(),
    }
}
