use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::charset::Charset;
use crate::one::property::color_ref::ColorRef;
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::paragraph_alignment::ParagraphAlignment;
use crate::one::property_set::{
    embedded_ink_container, math_inline_object, paragraph_style_object, rich_text_node,
    text_run_data,
};
use crate::onenote::ParserContext;
use crate::onenote::ink::{Ink, InkBoundingBox, InkContent, parse_ink_data};
use crate::onenote::math_inline_object::{MathInlineObject, parse_math_inline_object};
use crate::onenote::note_tag::{NoteTag, parse_note_tags};
use crate::onestore::ObjectSpace;
use itertools::Itertools;

/// A rich text paragraph.
///
/// # Formatting
///
/// Rich-text formatting is represented by storing the paragraph text along
/// with a list of text runs. Each text run specified formatting that is only
/// applied to a substring of the paragraph text.
///
/// The text run indices represent where each text run ends. The last text run
/// always ends at the end of the paragraph text. If there are no text run indices,
/// the text run formatting applies to the whole paragraph.
///
/// Text runs can be rendered by splitting the paragraph text at the text run
/// indices and then applying each text run formatting to its respective
/// substring.
#[derive(Clone, Debug)]
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
    pub(crate) embedded_objects: Vec<EmbeddedObject>,
    pub(crate) math_inline_objects: Vec<MathInlineObject>,
}

/// A hyperlink attached to a visible range of rich text.
///
/// OneNote stores the destination in a hidden `HYPERLINK` marker immediately
/// before one or more visible hyperlink-formatted runs. The range uses the
/// same UTF-16 code-unit offsets as [`RichText::text_run_indices`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextHyperlink {
    target: String,
    start: u32,
    end: u32,
}

impl TextHyperlink {
    /// The destination exactly as stored by OneNote.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Inclusive UTF-16 start offset of the visible link text.
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Exclusive UTF-16 end offset of the visible link text.
    pub fn end(&self) -> u32 {
        self.end
    }
}

impl RichText {
    /// The paragraph text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The formatting of each text run.
    ///
    /// See [\[MS-ONE\] 2.3.77].
    ///
    /// [\[MS-ONE\] 2.3.77]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/7b560477-14d0-4f2c-a65e-4159f69f299f
    pub fn text_run_formatting(&self) -> &[ParagraphStyling] {
        &self.text_run_formatting
    }

    /// The character positions where the text runs end.
    ///
    /// See [\[MS-ONE\] 2.3.76].
    ///
    /// [\[MS-ONE\] 2.3.76]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/f5ae3d7a-09dd-4904-a8bd-7a529d8067c3
    pub fn text_run_indices(&self) -> &[u32] {
        &self.text_run_indices
    }

    /// Return hyperlinks associated with visible text ranges.
    ///
    /// Malformed or unassociated hidden markers are ignored. Their source text
    /// and formatting remain available through the ordinary rich-text APIs.
    pub fn hyperlinks(&self) -> Vec<TextHyperlink> {
        extract_hyperlinks(
            &self.text,
            &self.text_run_indices,
            &self.text_run_formatting,
        )
    }

    /// The base paragraph style.
    pub fn paragraph_style(&self) -> &ParagraphStyling {
        &self.paragraph_style
    }

    /// The paragraph's top margin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.81].
    ///
    /// [\[MS-ONE\] 2.3.81]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/1a2958bd-7512-419b-a8a5-eda200edb7cd
    pub fn paragraph_space_before(&self) -> f32 {
        self.paragraph_space_before
    }

    /// The paragraph's bottom margin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.82].
    ///
    /// [\[MS-ONE\] 2.3.82]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/505393e2-c641-416f-be83-050da44d581d
    pub fn paragraph_space_after(&self) -> f32 {
        self.paragraph_space_after
    }

    /// The paragraph's line spacing in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.83].
    ///
    /// [\[MS-ONE\] 2.3.83]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/4474bd74-5407-4675-a9bb-a32f81eb799c
    pub fn paragraph_line_spacing_exact(&self) -> Option<f32> {
        self.paragraph_line_spacing_exact
    }

    /// The paragraph's text alignment.
    ///
    /// See [\[MS-ONE\] 2.3.94].
    ///
    /// [\[MS-ONE\] 2.3.94]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/36edb135-5e8e-400f-9394-82853d662d90
    pub fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.paragraph_alignment
    }

    /// The paragraph's alignment relative to the containing outline element (if present).
    ///
    /// See [\[MS-ONE\] 2.3.27].
    ///
    /// [\[MS-ONE\] 2.3.27]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/61fa50be-c355-4b8d-ac01-761a2f7f66c0
    pub fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    /// The paragraph's alignment.
    ///
    /// See [\[MS-ONE\] 2.3.33].
    ///
    /// [\[MS-ONE\] 2.3.33]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/4e7fe9db-2fdb-4239-b291-dc4b909c94ad
    pub fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    /// Note tags for this paragraph.
    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }

    /// Objects embedded in this paragraph.
    pub fn embedded_objects(&self) -> &[EmbeddedObject] {
        &self.embedded_objects
    }

    /// Math inline objects embedded in this paragraph.
    pub fn math_inline_objects(&self) -> &[MathInlineObject] {
        &self.math_inline_objects
    }
}

/// An object embedded in a rich text paragraph.
#[derive(Clone, Debug)]
pub enum EmbeddedObject {
    /// An ink handwriting object container.
    Ink(EmbeddedInkContainer),

    /// A space in the ink handwriting.
    InkSpace(EmbeddedInkSpace),

    /// A line break in the ink handwriting.
    InkLineBreak,
}

/// An ink handwriting object container.
#[derive(Clone, Debug)]
pub struct EmbeddedInkContainer {
    pub(crate) ink: Ink,
    pub(crate) bounding_box: Option<InkBoundingBox>,
}

impl EmbeddedInkContainer {
    /// The ink data embedded in a paragraph.
    pub fn ink(&self) -> &Ink {
        &self.ink
    }

    /// The ink object's bounding box.
    pub fn bounding_box(&self) -> Option<&InkBoundingBox> {
        self.bounding_box.as_ref()
    }
}

/// A space in an embedded ink handwriting object.
#[derive(Clone, Debug)]
pub struct EmbeddedInkSpace {
    height: f32,
    width: f32,
}

impl EmbeddedInkSpace {
    /// The space's height.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The space's width.
    pub fn width(&self) -> f32 {
        self.width
    }
}

/// A paragraph's style.
///
/// See [\[MS-ONE\] 2.2.43] and [\[MS-ONE\] 2.2.44].
///
/// [\[MS-ONE\] 2.2.43]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/38eb9b74-cfaf-4df7-b061-a83968c7ff5b
/// [\[MS-ONE\] 2.2.44]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/f0baabae-f42a-42e0-8cb2-869d420e865f
#[derive(Clone, Debug, Default)]
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
    pub(crate) hyperlink_protected: bool,
    pub(crate) hidden: bool,
}

impl ParagraphStyling {
    /// The text's charset.
    pub fn charset(&self) -> Option<Charset> {
        self.charset
    }

    /// Whether the text is bold.
    pub fn bold(&self) -> bool {
        self.bold
    }

    /// Whether the text is italic.
    pub fn italic(&self) -> bool {
        self.italic
    }

    /// Whether the text is underlined.
    pub fn underline(&self) -> bool {
        self.underline
    }

    /// Whether the text has strike-through formatting.
    pub fn strikethrough(&self) -> bool {
        self.strikethrough
    }

    /// Whether the text is formatted as superscript.
    pub fn superscript(&self) -> bool {
        self.superscript
    }

    /// Whether the text is formatted as subscript.
    pub fn subscript(&self) -> bool {
        self.subscript
    }

    /// The font for this text.
    pub fn font(&self) -> Option<&str> {
        self.font.as_deref()
    }

    /// The font size for this text in half-point increments.
    ///
    /// See [\[MS-ONE\] 2.3.16].
    ///
    /// [\[MS-ONE\] 2.3.16]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/f209fd9c-9042-4df2-b90c-1be20ac9c2d3
    pub fn font_size(&self) -> Option<u16> {
        self.font_size
    }

    /// The font color for this text.
    ///
    /// See [\[MS-ONE\] 2.3.45].
    ///
    /// [\[MS-ONE\] 2.3.45]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/17a7e6a7-7fa9-456f-a3fe-b2d8fef31be3
    pub fn font_color(&self) -> Option<ColorRef> {
        self.font_color
    }

    /// The background color for this text.
    ///
    /// See [\[MS-ONE\] 2.3.6].
    ///
    /// [\[MS-ONE\] 2.3.6]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/9932eafb-8200-4fc6-aa86-3a4d6a60bb62
    pub fn highlight(&self) -> Option<ColorRef> {
        self.highlight
    }

    /// The name of the default style for the next paragraph.
    ///
    /// See [\[MS-ONE\] 2.2.92].
    ///
    /// [\[MS-ONE\] 2.2.92]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/fa70a53b-2661-4d33-aec7-948488e49fc3
    pub fn next_style(&self) -> Option<&str> {
        self.next_style.as_deref()
    }

    /// The paragraph style's name.
    ///
    /// See [\[MS-ONE\] 2.2.83].
    ///
    /// [\[MS-ONE\] 2.2.83]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/4c9ad3ed-d804-44df-9c49-55b2a867db66
    pub fn style_id(&self) -> Option<&str> {
        self.style_id.as_deref()
    }

    /// The paragraph alignment.
    pub fn paragraph_alignment(&self) -> Option<ParagraphAlignment> {
        self.paragraph_alignment
    }

    /// The paragraph's top margin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.81].
    ///
    /// [\[MS-ONE\] 2.3.81]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/1a2958bd-7512-419b-a8a5-eda200edb7cd
    pub fn paragraph_space_before(&self) -> Option<f32> {
        self.paragraph_space_before
    }

    /// The paragraph's bottom margin in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.82].
    ///
    /// [\[MS-ONE\] 2.3.82]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/505393e2-c641-416f-be83-050da44d581d
    pub fn paragraph_space_after(&self) -> Option<f32> {
        self.paragraph_space_after
    }

    /// The paragraph's line spacing in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.83].
    ///
    /// [\[MS-ONE\] 2.3.83]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/4474bd74-5407-4675-a9bb-a32f81eb799c
    pub fn paragraph_line_spacing_exact(&self) -> Option<f32> {
        self.paragraph_line_spacing_exact
    }

    /// The LCID language code for the text.
    ///
    /// See [\[MS-ONE\] 2.3.26].
    ///
    /// [\[MS-ONE\] 2.3.26]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/f82cdbc0-d4e9-4cd0-bd0b-8c7734853d7f
    pub fn language_code(&self) -> Option<u32> {
        self.language_code
    }

    /// Whether the text is formatted as a math expression
    pub fn math_formatting(&self) -> bool {
        self.math_formatting
    }

    /// Whether the text is part of a hyperlink (the `\u{fddf}HYPERLINK "URL"`
    /// marker run carries this; the visible display text run also carries it
    /// alongside [`hyperlink_protected`](Self::hyperlink_protected)).
    ///
    /// See [\[MS-ONE\] 2.3.75].
    ///
    /// [\[MS-ONE\] 2.3.75]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/0464fc15-edd2-4f80-9d8a-3e892eea6dad
    pub fn hyperlink(&self) -> bool {
        self.hyperlink
    }

    /// Whether the text run is the visible display text for a hyperlink
    /// (as opposed to the hidden `\u{fddf}HYPERLINK "URL"` marker run that
    /// precedes it).
    ///
    /// See [\[MS-ONE\] 2.3.77].
    ///
    /// [\[MS-ONE\] 2.3.77]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/9bbc1f7a-9c85-46f2-9b27-7d4f9f1ec9b8
    pub fn hyperlink_protected(&self) -> bool {
        self.hyperlink_protected
    }

    /// Whether the text run is hidden from display (used for the marker
    /// portion of inline hyperlinks).
    ///
    /// See [\[MS-ONE\] 2.3.76].
    ///
    /// [\[MS-ONE\] 2.3.76]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/9b67d41f-8c4f-43e6-86b3-9b3d7a1f0f5a
    pub fn hidden(&self) -> bool {
        self.hidden
    }
}

// Embedded object types
const INK_SPACE_BLOB: u32 = 0x00020026;
const INK_END_OF_LINE_BLOB: u32 = 0x00020027;

pub(crate) fn parse_rich_text(
    content_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<RichText> {
    let object = space
        .get_object(content_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("rich text content is missing".into()))?;
    let data = rich_text_node::parse(object)?;

    // Parse the base paragraph style
    let paragraph_style = if let Some(paragraph_style_id) = data.paragraph_style {
        let paragraph_style_object = space.get_object(paragraph_style_id).ok_or_else(|| {
            ErrorKind::MalformedOneNoteData("paragraph styling is missing".into())
        })?;
        let paragraph_style_data = paragraph_style_object::parse(paragraph_style_object, ctx)?;
        parse_style(paragraph_style_data)
    } else {
        warn!(ctx, "rich text has no paragraph style; using defaults");
        ParagraphStyling::default()
    };

    // Parse the styles text runs (part 1)
    let style_objects: Vec<_> = data
        .text_run_formatting
        .iter()
        .filter_map(|style_id| {
            space.get_object(*style_id).or_else(|| {
                warn!(ctx, "missing style for text run formatting: {:?}", style_id);

                None
            })
        })
        .collect();

    let styles_data: Vec<paragraph_style_object::Data> = style_objects
        .into_iter()
        .map(|style_object| paragraph_style_object::parse(style_object, ctx))
        .collect::<Result<Vec<_>>>()?;

    // Parse text run data
    let text_run_data = text_run_data::parse(object)?.unwrap_or_default();

    // Parse math text runs
    let math_inline_objects = text_run_data
        .iter()
        .zip(&styles_data)
        .map(|(text_run_data, style_data)| {
            if style_data.math_formatting {
                let math_data = math_inline_object::Data::parse(text_run_data)?;
                let math_inline_object = parse_math_inline_object(math_data)?;

                return Ok(Some(math_inline_object));
            }

            Ok(None)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect_vec();

    // Parse the embedded objects
    let objects = text_run_data
        .into_iter()
        .zip(&styles_data)
        .map(|(embedded_object, style_data)| {
            if style_data.text_run_is_embedded_object {
                let object_data = embedded_ink_container::Data::parse(embedded_object)?;
                return Ok(Some((style_data.text_run_object_type, object_data)));
            }

            Ok(None)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect_vec();

    let mut objects_without_ref = 0;

    let embedded_objects: Vec<_> = objects
        .into_iter()
        .enumerate()
        .map(|(i, (object_type, embedded_data))| {
            let i = i - objects_without_ref;

            let object_ref = data.text_run_data_object.get(i);
            let is_valid_ref = object_ref
                .map(|object_ref| space.get_object(*object_ref).is_some())
                .unwrap_or(true);

            // Based on sample .one files, spaces and EOL blobs either:
            // - have an invalid associated object reference (is_valid_ref = false), or
            // - have no associated object reference (is_valid_ref = true).
            //
            // In the first case, the object reference is skipped automatically.
            // In the second, we adjust so references for subsequent objects aren't
            // shifted.
            if let Some(object_type) = object_type {
                if is_valid_ref
                    && (object_type == INK_END_OF_LINE_BLOB || object_type == INK_SPACE_BLOB)
                {
                    objects_without_ref += 1;
                }
            }

            match object_type {
                Some(INK_END_OF_LINE_BLOB) => Ok(Some(EmbeddedObject::InkLineBreak)),
                Some(INK_SPACE_BLOB) => parse_embedded_ink_space(embedded_data)
                    .map(|space_obj| Some(EmbeddedObject::InkSpace(space_obj))),
                None => {
                    if let Some(object_ref) = object_ref {
                        return parse_embedded_ink_data(*object_ref, space, embedded_data, ctx)
                            .map(|container| Some(EmbeddedObject::Ink(container)));
                    }

                    Ok(None)
                }
                Some(v) => Err(ErrorKind::MalformedOneNoteFileData(
                    format!("unknown embedded object type: {:x}", v).into(),
                )
                .into()),
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect_vec();

    // Parse the styles text runs (part 2)
    let mut styles = styles_data.into_iter().map(parse_style).collect_vec();

    let mut text = if !embedded_objects.is_empty() {
        "".to_string()
    } else {
        data.text.unwrap_or_default()
    };

    let mut text_run_indices = data.text_run_indices;

    fix_leading_vt_misalignment(&mut text, &mut text_run_indices, &mut styles);

    let text = RichText {
        text,
        embedded_objects,
        text_run_formatting: styles,
        text_run_indices,
        paragraph_style,
        paragraph_space_before: data.paragraph_space_before,
        paragraph_space_after: data.paragraph_space_after,
        paragraph_line_spacing_exact: data.paragraph_line_spacing_exact,
        paragraph_alignment: data.paragraph_alignment,
        layout_alignment_in_parent: data.layout_alignment_in_parent,
        layout_alignment_self: data.layout_alignment_self,
        note_tags: parse_note_tags(&data.note_tags, space)?,
        math_inline_objects,
    };

    Ok(text)
}

fn parse_embedded_ink_data(
    embedded_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    data: embedded_ink_container::Data,
    ctx: &mut ParserContext,
) -> Result<EmbeddedInkContainer> {
    let (strokes, bb) = parse_ink_data(embedded_id, space, None, None, ctx)?;

    let display_bb = data
        .start_x
        .zip(data.start_y)
        .zip(data.height)
        .zip(data.width)
        .map(|(((x, y), height), width)| InkBoundingBox {
            x,
            y,
            height,
            width,
        });

    let data = EmbeddedInkContainer {
        ink: Ink {
            content: InkContent::Strokes(strokes),
            bounding_box: bb,
            offset_horizontal: data.offset_horiz,
            offset_vertical: data.offset_vert,
        },
        bounding_box: display_bb,
    };

    Ok(data)
}

fn parse_embedded_ink_space(data: embedded_ink_container::Data) -> Result<EmbeddedInkSpace> {
    let width = data.space_width.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("embedded ink space has no width".into())
    })?;

    let height = data.space_height.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("embedded ink space has no height".into())
    })?;

    Ok(EmbeddedInkSpace { height, width })
}

// Workaround for an undiagnosed OneNote writer quirk: the paragraph text begins
// with U+000B (vertical tab) and a corresponding leading TextRunIndex of 1, but
// TextRunFormatting is built as if that leading VT segment did not exist. Every
// styling slot ends up paired with the run before it — prose ends up flagged
// Hyperlink/HyperlinkProtected/Hidden, link display text ends up plain — and
// one trailing styling slot is unused.
fn fix_leading_vt_misalignment(
    text: &mut String,
    indices: &mut Vec<u32>,
    styles: &mut Vec<ParagraphStyling>,
) {
    if !text.starts_with('\u{000B}') {
        return;
    }
    if indices.first() != Some(&1) {
        return;
    }

    // Two variants of the same OneNote quirk are observed in the wild:
    //
    //   A) `len(styles) == len(indices) + 1` (spec-conformant on its face).
    //      OneNote also wrote one phantom trailing styling slot that mirrors
    //      the leading VT split, so after we drop the VT split we must also
    //      drop that trailing styling slot to restore alignment.
    //
    //   B) `len(styles) == len(indices)`. OneNote omitted the trailing styling
    //      slot entirely, leaving the input non-conformant. Dropping the VT
    //      split alone makes it spec-conformant — no styling pop needed.
    //
    // Anything else (e.g. styles much longer or shorter) is unfamiliar; bail
    // out rather than rewrite blindly.
    let drop_trailing_style = match styles.len().checked_sub(indices.len()) {
        Some(1) => true,
        Some(0) => false,
        _ => return,
    };

    text.remove(0);
    indices.remove(0);
    for idx in indices.iter_mut() {
        *idx -= 1;
    }
    if drop_trailing_style {
        styles.pop();
    }
}

const HYPERLINK_MARKER: &str = "\u{fddf}HYPERLINK \"";

fn extract_hyperlinks(
    text: &str,
    indices: &[u32],
    styles: &[ParagraphStyling],
) -> Vec<TextHyperlink> {
    let total = u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX);
    let runs = text_runs(total, indices, styles);
    let mut links = Vec::new();
    let mut search_from = 0;

    while let Some(relative_start) = text[search_from..].find(HYPERLINK_MARKER) {
        let marker_start = search_from + relative_start;
        let target_start = marker_start + HYPERLINK_MARKER.len();
        let Some(relative_end) = text[target_start..].find('"') else {
            break;
        };
        let target_end = target_start + relative_end;
        let marker_end = target_end + '"'.len_utf8();
        search_from = marker_end;

        let marker_start_utf16 = byte_to_utf16(text, marker_start);
        let marker_end_utf16 = byte_to_utf16(text, marker_end);
        let overlapping = runs
            .iter()
            .filter(|run| run.end > marker_start_utf16 && run.start < marker_end_utf16)
            .collect::<Vec<_>>();
        let marker_is_hidden_link = !overlapping.is_empty()
            && overlapping
                .iter()
                .all(|run| run.style.hyperlink && run.style.hidden);
        if !marker_is_hidden_link {
            continue;
        }

        let mut first = None;
        for (index, run) in runs.iter().enumerate() {
            if run.end <= marker_end_utf16 {
                continue;
            }
            if run.style.hidden && run.style.hyperlink {
                continue;
            }
            if run.style.hyperlink && !run.style.hidden {
                first = Some(index);
            }
            break;
        }
        let Some(first) = first else {
            continue;
        };
        let start = runs[first].start.max(marker_end_utf16);
        let mut end = runs[first].end;
        for run in &runs[first + 1..] {
            if run.start != end || !run.style.hyperlink || run.style.hidden {
                break;
            }
            end = run.end;
        }
        if start < end && target_start < target_end {
            links.push(TextHyperlink {
                target: text[target_start..target_end].to_owned(),
                start,
                end,
            });
        }
    }

    links
}

struct TextRun<'a> {
    start: u32,
    end: u32,
    style: &'a ParagraphStyling,
}

fn text_runs<'a>(total: u32, indices: &[u32], styles: &'a [ParagraphStyling]) -> Vec<TextRun<'a>> {
    let mut start = 0;
    styles
        .iter()
        .enumerate()
        .map(|(index, style)| {
            let end = indices
                .get(index)
                .copied()
                .unwrap_or(total)
                .min(total)
                .max(start);
            let run = TextRun { start, end, style };
            start = end;
            run
        })
        .collect()
}

fn byte_to_utf16(text: &str, byte: usize) -> u32 {
    u32::try_from(text[..byte].encode_utf16().count()).unwrap_or(u32::MAX)
}

fn parse_style(data: paragraph_style_object::Data) -> ParagraphStyling {
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
        hyperlink_protected: data.hyperlink_protected,
        hidden: data.hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::{HYPERLINK_MARKER, ParagraphStyling, extract_hyperlinks};

    fn style(hyperlink: bool, hidden: bool) -> ParagraphStyling {
        ParagraphStyling {
            hyperlink,
            hyperlink_protected: hyperlink,
            hidden,
            ..ParagraphStyling::default()
        }
    }

    #[test]
    fn extracts_multiple_visible_hyperlink_ranges() {
        let first_marker = format!("{HYPERLINK_MARKER}https://example.test/one\"");
        let second_marker = format!("{HYPERLINK_MARKER}mailto:user@example.test\"");
        let text = format!("{first_marker}first and {second_marker}second");
        let first_marker_end = u32::try_from(first_marker.encode_utf16().count()).unwrap();
        let first_text_end = first_marker_end + 5;
        let plain_end = first_text_end + 5;
        let second_marker_end =
            plain_end + u32::try_from(second_marker.encode_utf16().count()).unwrap();
        let total = u32::try_from(text.encode_utf16().count()).unwrap();
        let indices = vec![
            first_marker_end,
            first_text_end,
            plain_end,
            second_marker_end,
        ];
        let styles = vec![
            style(true, true),
            style(true, false),
            style(false, false),
            style(true, true),
            style(true, false),
        ];

        let links = extract_hyperlinks(&text, &indices, &styles);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target(), "https://example.test/one");
        assert_eq!(
            (links[0].start(), links[0].end()),
            (first_marker_end, first_text_end)
        );
        assert_eq!(links[1].target(), "mailto:user@example.test");
        assert_eq!(
            (links[1].start(), links[1].end()),
            (second_marker_end, total)
        );
    }

    #[test]
    fn supports_markers_split_across_hidden_runs_and_unicode_display_text() {
        let marker = format!("{HYPERLINK_MARKER}onenote:#page-id={{abc}}\"");
        let text = format!("{marker}風景");
        let marker_end = u32::try_from(marker.encode_utf16().count()).unwrap();
        let total = u32::try_from(text.encode_utf16().count()).unwrap();
        let indices = vec![1, marker_end];
        let styles = vec![style(true, true), style(true, true), style(true, false)];

        let links = extract_hyperlinks(&text, &indices, &styles);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target(), "onenote:#page-id={abc}");
        assert_eq!((links[0].start(), links[0].end()), (marker_end, total));
    }

    #[test]
    fn ignores_malformed_and_unstyled_markers() {
        let malformed = format!("{HYPERLINK_MARKER}https://example.test");
        assert!(extract_hyperlinks(&malformed, &[], &[]).is_empty());

        let complete = format!("{HYPERLINK_MARKER}https://example.test\"label");
        assert!(extract_hyperlinks(&complete, &[], &[style(false, false)]).is_empty());
    }
}
