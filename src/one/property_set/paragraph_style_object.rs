use crate::one::property::charset::Charset;
use crate::one::property::color_ref::ColorRef;
use crate::one::property::paragraph_alignment::ParagraphAlignment;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Data {
    charset: Option<Charset>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    superscript: bool,
    subscript: bool,
    font: Option<String>,
    font_size: Option<u16>,
    font_color: Option<ColorRef>,
    highlight: Option<ColorRef>,
    next_style: Option<String>,
    style_id: Option<String>,
    paragraph_alignment: Option<ParagraphAlignment>,
    paragraph_space_before: Option<f32>,
    paragraph_space_after: Option<f32>,
    paragraph_line_spacing_exact: Option<f32>,
    language_code: Option<u32>,
    math_formatting: bool,
    hyperlink: bool,
    hyperlink_protected: bool,
    hidden: bool,
    text_run_is_embedded_object: bool,
}

impl Data {
    pub(crate) fn charset(&self) -> Option<Charset> {
        self.charset
    }

    pub(crate) fn bold(&self) -> bool {
        self.bold
    }

    pub(crate) fn italic(&self) -> bool {
        self.italic
    }

    pub(crate) fn underline(&self) -> bool {
        self.underline
    }

    pub(crate) fn strikethrough(&self) -> bool {
        self.strikethrough
    }

    pub(crate) fn superscript(&self) -> bool {
        self.superscript
    }

    pub(crate) fn subscript(&self) -> bool {
        self.subscript
    }

    pub(crate) fn font(&self) -> Option<&str> {
        self.font.as_deref()
    }

    pub(crate) fn font_size(&self) -> Option<u16> {
        self.font_size
    }

    pub(crate) fn font_color(&self) -> Option<ColorRef> {
        self.font_color
    }

    pub(crate) fn highlight(&self) -> Option<ColorRef> {
        self.highlight
    }

    pub(crate) fn next_style(&self) -> Option<&str> {
        self.next_style.as_deref()
    }

    pub(crate) fn style_id(&self) -> Option<&str> {
        self.style_id.as_deref()
    }

    pub(crate) fn paragraph_alignment(&self) -> Option<ParagraphAlignment> {
        self.paragraph_alignment
    }

    pub(crate) fn paragraph_space_before(&self) -> Option<f32> {
        self.paragraph_space_before
    }

    pub(crate) fn paragraph_space_after(&self) -> Option<f32> {
        self.paragraph_space_after
    }

    pub(crate) fn paragraph_line_spacing_exact(&self) -> Option<f32> {
        self.paragraph_line_spacing_exact
    }

    pub(crate) fn language_code(&self) -> Option<u32> {
        self.language_code
    }

    pub(crate) fn math_formatting(&self) -> bool {
        self.math_formatting
    }

    pub(crate) fn hyperlink(&self) -> bool {
        self.hyperlink
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::ParagraphStyleObject.as_jcid());

    let charset = Charset::parse(PropertyType::Charset, object);
    let bold = simple::parse_bool(PropertyType::Bold, object).unwrap_or_default();
    let italic = simple::parse_bool(PropertyType::Italic, object).unwrap_or_default();
    let underline = simple::parse_bool(PropertyType::Underline, object).unwrap_or_default();
    let strikethrough = simple::parse_bool(PropertyType::Strikethrough, object).unwrap_or_default();
    let superscript = simple::parse_bool(PropertyType::Superscript, object).unwrap_or_default();
    let subscript = simple::parse_bool(PropertyType::Subscript, object).unwrap_or_default();
    let font = simple::parse_string(PropertyType::Font, object);
    let font_size = simple::parse_u16(PropertyType::FontSize, object);
    let font_color = ColorRef::parse(PropertyType::FontColor, object);
    let highlight = ColorRef::parse(PropertyType::Highlight, object);
    let next_style = simple::parse_string(PropertyType::NextStyle, object);
    let style_id = simple::parse_string(PropertyType::ParagraphStyleId, object);
    let paragraph_alignment = ParagraphAlignment::parse(object);
    let paragraph_space_before = simple::parse_f32(PropertyType::ParagraphSpaceBefore, object);
    let paragraph_space_after = simple::parse_f32(PropertyType::ParagraphSpaceAfter, object);
    let paragraph_line_spacing_exact =
        simple::parse_f32(PropertyType::ParagraphLineSpacingExact, object);
    let language_code = simple::parse_u32(PropertyType::LanguageID, object);
    let math_formatting =
        simple::parse_bool(PropertyType::MathFormatting, object).unwrap_or_default();
    let hyperlink = simple::parse_bool(PropertyType::Hyperlink, object).unwrap_or_default();
    let hyperlink_protected =
        simple::parse_bool(PropertyType::HyperlinkProtected, object).unwrap_or_default();
    let hidden = simple::parse_bool(PropertyType::Hidden, object).unwrap_or_default();
    let text_run_is_embedded_object =
        simple::parse_bool(PropertyType::TextRunIsEmbeddedObject, object).unwrap_or_default();

    Data {
        charset,
        bold,
        italic,
        underline,
        strikethrough,
        superscript,
        subscript,
        font,
        font_size,
        font_color,
        highlight,
        next_style,
        style_id,
        paragraph_alignment,
        paragraph_space_before,
        paragraph_space_after,
        paragraph_line_spacing_exact,
        language_code,
        math_formatting,
        hyperlink,
        hyperlink_protected,
        hidden,
        text_run_is_embedded_object,
    }
}
