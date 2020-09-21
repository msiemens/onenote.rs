use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::note_tag_state::NoteTagState;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified: Time,
    picture_container: Option<ExGuid>,
    layout_max_width: f32,
    layout_max_height: f32,
    is_layout_size_set_by_user: bool,
    language_code: Option<u32>,
    alt_text: Option<String>,
    layout_alignment_in_parent: Option<LayoutAlignment>,
    layout_alignment_self: Option<LayoutAlignment>,
    image_filename: Option<String>,
    displayed_page_number: Option<u32>,
    text: Option<String>,
    text_language_code: Option<u32>,
    picture_width: Option<f32>,
    picture_height: Option<f32>,
    hyperlink_url: Option<String>,
    note_tag_states: Vec<NoteTagState>,
    offset_from_parent_horiz: Option<f32>,
    offset_from_parent_vert: Option<f32>,
    is_background: bool,
}

impl Data {
    pub(crate) fn picture_container(&self) -> Option<ExGuid> {
        self.picture_container
    }

    pub(crate) fn layout_max_width(&self) -> f32 {
        self.layout_max_width
    }

    pub(crate) fn layout_max_height(&self) -> f32 {
        self.layout_max_height
    }

    pub(crate) fn alt_text(&self) -> Option<&str> {
        self.alt_text.as_deref()
    }

    pub(crate) fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub(crate) fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    pub(crate) fn image_filename(&self) -> Option<&str> {
        self.image_filename.as_deref()
    }

    pub(crate) fn displayed_page_number(&self) -> Option<u32> {
        self.displayed_page_number
    }

    pub(crate) fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub(crate) fn text_language_code(&self) -> Option<u32> {
        self.text_language_code
    }

    pub(crate) fn picture_width(&self) -> Option<f32> {
        self.picture_width
    }

    pub(crate) fn picture_height(&self) -> Option<f32> {
        self.picture_height
    }

    pub(crate) fn hyperlink_url(&self) -> Option<&str> {
        self.hyperlink_url.as_deref()
    }

    pub(crate) fn offset_from_parent_horiz(&self) -> Option<f32> {
        self.offset_from_parent_horiz
    }

    pub(crate) fn offset_from_parent_vert(&self) -> Option<f32> {
        self.offset_from_parent_vert
    }

    pub(crate) fn is_background(&self) -> bool {
        self.is_background
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::ImageNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("image has no last modified time");
    let picture_container = ObjectReference::parse(PropertyType::PictureContainer, object);
    let layout_max_width = simple::parse_f32(PropertyType::LayoutMaxWidth, object)
        .expect("image has no layout max width");
    let layout_max_height = simple::parse_f32(PropertyType::LayoutMaxHeight, object)
        .expect("image has no layout max height");
    let is_layout_size_set_by_user =
        simple::parse_bool(PropertyType::IsLayoutSizeSetByUser, object).unwrap_or_default();
    let language_code = simple::parse_u32(PropertyType::LanguageID, object);
    let alt_text = simple::parse_string(PropertyType::ImageAltText, object);
    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object);
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object);
    let image_filename = simple::parse_string(PropertyType::ImageFilename, object);
    let displayed_page_number = simple::parse_u32(PropertyType::DisplayedPageNumber, object);
    let text = simple::parse_string(PropertyType::RichEditTextUnicode, object);
    let text_language_code =
        simple::parse_u16(PropertyType::RichEditTextLangID, object).map(|value| value as u32);
    let picture_width = simple::parse_f32(PropertyType::PictureWidth, object);
    let picture_height = simple::parse_f32(PropertyType::PictureHeight, object);
    let hyperlink_url = simple::parse_string(PropertyType::WzHyperlinkUrl, object);
    let offset_from_parent_horiz = simple::parse_f32(PropertyType::OffsetFromParentHoriz, object);
    let offset_from_parent_vert = simple::parse_f32(PropertyType::OffsetFromParentVert, object);
    let is_background = simple::parse_bool(PropertyType::IsBackground, object).unwrap_or_default();

    Data {
        last_modified,
        picture_container,
        layout_max_width,
        layout_max_height,
        is_layout_size_set_by_user,
        language_code,
        alt_text,
        layout_alignment_in_parent,
        layout_alignment_self,
        image_filename,
        displayed_page_number,
        text,
        text_language_code,
        picture_width,
        picture_height,
        hyperlink_url,
        note_tag_states: vec![], // FIXME: Parse this
        offset_from_parent_horiz,
        offset_from_parent_vert,
        is_background,
    }
}
