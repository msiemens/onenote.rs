use crate::one::property::charset::Charset;
use crate::one::property::color_ref::ColorRef;
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::paragraph_alignment::ParagraphAlignment;
use crate::one::property_set::{paragraph_style_object, rich_text_node};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct RichText {
    pub(crate) text: String,

    pub(crate) text_run_formatting: Vec<ParagraphStyling>,
    pub(crate) text_run_indices: Vec<u32>,

    pub(crate) paragraph_style: ParagraphStyling,
    pub(crate) paragraph_space_before: f32,
    pub(crate) paragraph_space_after: f32,
    pub(crate) paragraph_line_spacing_exact: Option<f32>,
    pub(crate) paragraph_alignment: ParagraphAlignment,

    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,
}

#[derive(Debug)]
pub struct ParagraphStyling {
    pub(crate) charset: Option<Charset>,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) superscript: bool,
    pub(crate) subscript: bool,
    pub(crate) font: Option<String>,
    pub(crate) font_size: Option<u16>,
    pub(crate) font_color: Option<ColorRef>,
    pub(crate) highlight: Option<ColorRef>,
    pub(crate) next_style: Option<String>,
    pub(crate) style_id: Option<String>,
    pub(crate) paragraph_alignment: Option<ParagraphAlignment>,
    pub(crate) paragraph_space_before: Option<f32>,
    pub(crate) paragraph_space_after: Option<f32>,
    pub(crate) paragraph_line_spacing_exact: Option<f32>,
    pub(crate) language_code: Option<u32>,
    pub(crate) math_formatting: bool,
    pub(crate) hyperlink: bool,
}

pub(crate) fn parse_rich_text(content_id: ExGuid, space: &ObjectSpace) -> RichText {
    let object = space
        .get_object(content_id)
        .expect("rich text content is missing");
    let data = rich_text_node::parse(object);

    let style = parse_style(data.paragraph_style(), space);

    let styles = data
        .text_run_formatting()
        .iter()
        .map(|id| parse_style(*id, space))
        .collect();

    // TODO: Parse lang code into iso code
    // dia-i18n = "0.8.0"

    RichText {
        text: data.text().unwrap_or_default().to_string(),
        text_run_formatting: styles,
        text_run_indices: data.text_run_indices().to_vec(),
        paragraph_style: style,
        paragraph_space_before: data.paragraph_space_before(),
        paragraph_space_after: data.paragraph_space_after(),
        paragraph_line_spacing_exact: data.paragraph_line_spacing_exact(),
        paragraph_alignment: data.paragraph_alignment(),
        layout_alignment_in_parent: data.layout_alignment_in_parent(),
        layout_alignment_self: data.layout_alignment_self(),
    }
}

fn parse_style(style_id: ExGuid, space: &ObjectSpace) -> ParagraphStyling {
    let object = space
        .get_object(style_id)
        .expect("paragraph styling is missing");
    let data = paragraph_style_object::parse(object);

    ParagraphStyling {
        charset: data.charset(),
        bold: data.bold(),
        italic: data.italic(),
        underline: data.underline(),
        strikethrough: data.strikethrough(),
        superscript: data.superscript(),
        subscript: data.subscript(),
        font: data.font().map(String::from),
        font_size: data.font_size(),
        font_color: data.font_color(),
        highlight: data.highlight(),
        next_style: data.next_style().map(String::from),
        style_id: data.style_id().map(String::from),
        paragraph_alignment: data.paragraph_alignment(),
        paragraph_space_before: data.paragraph_space_before(),
        paragraph_space_after: data.paragraph_space_after(),
        paragraph_line_spacing_exact: data.paragraph_line_spacing_exact(),
        language_code: data.language_code(),
        math_formatting: data.math_formatting(),
        hyperlink: data.hyperlink(),
    }
}
