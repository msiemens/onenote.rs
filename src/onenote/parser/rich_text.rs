use crate::one::property::charset::Charset;
use crate::one::property::color_ref::ColorRef;
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::paragraph_alignment::ParagraphAlignment;
use crate::one::property_set::{paragraph_style_object, rich_text_node};
use crate::onenote::parser::note_tag::{parse_note_tags, NoteTag};
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

    pub(crate) note_tags: Vec<NoteTag>,
}

impl RichText {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn text_run_formatting(&self) -> &[ParagraphStyling] {
        &self.text_run_formatting
    }

    pub fn text_run_indices(&self) -> &[u32] {
        &self.text_run_indices
    }

    pub fn paragraph_style(&self) -> &ParagraphStyling {
        &self.paragraph_style
    }

    pub fn paragraph_space_before(&self) -> f32 {
        self.paragraph_space_before
    }

    pub fn paragraph_space_after(&self) -> f32 {
        self.paragraph_space_after
    }

    pub fn paragraph_line_spacing_exact(&self) -> Option<f32> {
        self.paragraph_line_spacing_exact
    }

    pub fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.paragraph_alignment
    }

    pub fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }
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

impl ParagraphStyling {
    pub fn charset(&self) -> Option<Charset> {
        self.charset
    }

    pub fn bold(&self) -> bool {
        self.bold
    }

    pub fn italic(&self) -> bool {
        self.italic
    }

    pub fn underline(&self) -> bool {
        self.underline
    }

    pub fn strikethrough(&self) -> bool {
        self.strikethrough
    }

    pub fn superscript(&self) -> bool {
        self.superscript
    }

    pub fn subscript(&self) -> bool {
        self.subscript
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

    pub fn highlight(&self) -> Option<ColorRef> {
        self.highlight
    }

    pub fn next_style(&self) -> Option<&str> {
        self.next_style.as_deref()
    }

    pub fn style_id(&self) -> Option<&str> {
        self.style_id.as_deref()
    }

    pub fn paragraph_alignment(&self) -> Option<ParagraphAlignment> {
        self.paragraph_alignment
    }

    pub fn paragraph_space_before(&self) -> Option<f32> {
        self.paragraph_space_before
    }

    pub fn paragraph_space_after(&self) -> Option<f32> {
        self.paragraph_space_after
    }

    pub fn paragraph_line_spacing_exact(&self) -> Option<f32> {
        self.paragraph_line_spacing_exact
    }

    pub fn language_code(&self) -> Option<u32> {
        self.language_code
    }

    pub fn math_formatting(&self) -> bool {
        self.math_formatting
    }

    pub fn hyperlink(&self) -> bool {
        self.hyperlink
    }
}

pub(crate) fn parse_rich_text(content_id: ExGuid, space: &ObjectSpace) -> RichText {
    let object = space
        .get_object(content_id)
        .expect("rich text content is missing");
    let data = rich_text_node::parse(object);

    let style = parse_style(data.paragraph_style, space);

    let styles = data
        .text_run_formatting
        .into_iter()
        .map(|style_id| parse_style(style_id, space))
        .collect();

    // TODO: Parse lang code into iso code
    // dia-i18n = "0.8.0"

    RichText {
        text: data.text.unwrap_or_default(),
        text_run_formatting: styles,
        text_run_indices: data.text_run_indices,
        paragraph_style: style,
        paragraph_space_before: data.paragraph_space_before,
        paragraph_space_after: data.paragraph_space_after,
        paragraph_line_spacing_exact: data.paragraph_line_spacing_exact,
        paragraph_alignment: data.paragraph_alignment,
        layout_alignment_in_parent: data.layout_alignment_in_parent,
        layout_alignment_self: data.layout_alignment_self,
        note_tags: parse_note_tags(data.note_tags, space),
    }
}

fn parse_style(style_id: ExGuid, space: &ObjectSpace) -> ParagraphStyling {
    let object = space
        .get_object(style_id)
        .expect("paragraph styling is missing");
    let data = paragraph_style_object::parse(object);

    ParagraphStyling {
        charset: data.charset,
        bold: data.bold,
        italic: data.italic,
        underline: data.underline,
        strikethrough: data.strikethrough,
        superscript: data.superscript,
        subscript: data.subscript,
        font: data.font,
        font_size: data.font_size,
        font_color: data.font_color,
        highlight: data.highlight,
        next_style: data.next_style,
        style_id: data.style_id,
        paragraph_alignment: data.paragraph_alignment,
        paragraph_space_before: data.paragraph_space_before,
        paragraph_space_after: data.paragraph_space_after,
        paragraph_line_spacing_exact: data.paragraph_line_spacing_exact,
        language_code: data.language_code,
        math_formatting: data.math_formatting,
        hyperlink: data.hyperlink,
    }
}
