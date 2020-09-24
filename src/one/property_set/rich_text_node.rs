use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::paragraph_alignment::ParagraphAlignment;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::note_tag_container::Data as NoteTagData;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified_time: Time,
    tight_layout: bool,
    text_run_formatting: Vec<ExGuid>,
    text_run_indices: Vec<u32>,
    paragraph_style: ExGuid,
    paragraph_space_before: f32,
    paragraph_space_after: f32,
    paragraph_line_spacing_exact: Option<f32>,
    paragraph_alignment: ParagraphAlignment,
    text: Option<String>,
    is_title_time: bool,
    is_boiler_text: bool,
    is_title_date: bool,
    is_title_text: bool,
    layout_alignment_in_parent: Option<LayoutAlignment>,
    layout_alignment_self: Option<LayoutAlignment>,
    language_code: Option<u32>,
    rtl: bool,
    note_tags: Vec<NoteTagData>,
}

impl Data {
    pub(crate) fn text_run_formatting(&self) -> &[ExGuid] {
        &self.text_run_formatting
    }

    pub(crate) fn text_run_indices(&self) -> &[u32] {
        &self.text_run_indices
    }

    pub(crate) fn paragraph_style(&self) -> ExGuid {
        self.paragraph_style
    }

    pub(crate) fn paragraph_space_before(&self) -> f32 {
        self.paragraph_space_before
    }

    pub(crate) fn paragraph_space_after(&self) -> f32 {
        self.paragraph_space_after
    }

    pub(crate) fn paragraph_line_spacing_exact(&self) -> Option<f32> {
        self.paragraph_line_spacing_exact
    }

    pub(crate) fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.paragraph_alignment
    }

    pub(crate) fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub(crate) fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub(crate) fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    pub fn note_tags(&self) -> &[NoteTagData] {
        &self.note_tags
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::RichTextNode.as_jcid());

    let last_modified_time = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("rich text node has no last_modified time");
    let tight_layout =
        simple::parse_bool(PropertyType::LayoutTightLayout, object).unwrap_or_default();
    let text_run_formatting =
        ObjectReference::parse_vec(PropertyType::TextRunFormatting, object).unwrap_or_default();
    let text_run_indices =
        simple::parse_vec_u32(PropertyType::TextRunIndex, object).unwrap_or_default();
    let paragraph_style = ObjectReference::parse(PropertyType::ParagraphStyle, object)
        .expect("rich text has no paragraph style");
    let paragraph_space_before =
        simple::parse_f32(PropertyType::ParagraphSpaceBefore, object).unwrap_or_default();
    let paragraph_space_after =
        simple::parse_f32(PropertyType::ParagraphSpaceAfter, object).unwrap_or_default();
    let paragraph_line_spacing_exact =
        simple::parse_f32(PropertyType::ParagraphLineSpacingExact, object);
    let paragraph_alignment = ParagraphAlignment::parse(object).unwrap_or_default();

    let text = simple::parse_string(PropertyType::RichEditTextUnicode, object)
        .or_else(|| simple::parse_ascii(PropertyType::TextExtendedAscii, object));

    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object);
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object);

    let is_title_time = simple::parse_bool(PropertyType::IsTitleTime, object).unwrap_or_default();
    let is_boiler_text = simple::parse_bool(PropertyType::IsBoilerText, object).unwrap_or_default();
    let is_title_date = simple::parse_bool(PropertyType::IsTitleDate, object).unwrap_or_default();
    let is_title_text = simple::parse_bool(PropertyType::IsTitleText, object).unwrap_or_default();
    let language_code =
        simple::parse_u16(PropertyType::RichEditTextLangID, object).map(|value| value as u32);
    let rtl = simple::parse_bool(PropertyType::ReadingOrderRTL, object).unwrap_or_default();

    let note_tags = NoteTagData::parse(object).unwrap_or_default();

    Data {
        last_modified_time,
        tight_layout,
        text_run_formatting,
        text_run_indices,
        paragraph_style,
        paragraph_space_before,
        paragraph_space_after,
        paragraph_line_spacing_exact,
        paragraph_alignment,
        text,
        is_title_time,
        is_boiler_text,
        is_title_date,
        is_title_text,
        layout_alignment_in_parent,
        layout_alignment_self,
        language_code,
        rtl,
        note_tags,
    }
}
