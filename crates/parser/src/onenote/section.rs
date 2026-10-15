use crate::errors::{ErrorKind, Result};
use crate::one::property::color::Color;
use crate::one::property_set::{section_metadata_node, section_node};
use crate::onenote::ParserContext;
use crate::onenote::file_identity::FileIdentity;
use crate::onenote::page_series::{PageSeries, parse_page_series};
use crate::onestore::{ObjectSpace, OneStore};
use crate::warn::Report;

/// An entry in a section list.
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum SectionEntry {
    Section(Section),
    SectionGroup(SectionGroup),
}

/// A OneNote section.
///
/// See [\[MS-ONE\] 1.3.1] and [\[MS-ONE\] 2.2.17].
///
/// [\[MS-ONE\] 1.3.1]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/1603b29c-1c9f-4e85-b9b9-59684122374a
/// [\[MS-ONE\] 2.2.17]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/6913913f-b7d1-4b29-ab09-231ea3835ac2
#[derive(Clone, Debug)]
pub struct Section {
    file_identity: FileIdentity,
    display_name: String,
    page_series: Vec<PageSeries>,
    color: Option<Color>,
    pub(crate) report: Report,
}

impl Section {
    /// The identity stored in the section file.
    pub fn file_identity(&self) -> FileIdentity {
        self.file_identity
    }

    /// The section name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// The page series contained within the section.
    pub fn page_series(&self) -> &[PageSeries] {
        &self.page_series
    }

    /// The color of the section.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Non-fatal warnings encountered while parsing this section.
    pub fn report(&self) -> &Report {
        &self.report
    }
}

/// A group of sections.
#[derive(Clone, Debug)]
pub struct SectionGroup {
    pub(crate) file_identity: FileIdentity,
    pub(crate) display_name: String,
    pub(crate) entries: Vec<SectionEntry>,
}

impl SectionGroup {
    /// The identity stored in the section group's table-of-contents file.
    pub fn file_identity(&self) -> FileIdentity {
        self.file_identity
    }

    /// The group name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// The sections contained within the group.
    pub fn entries(&self) -> &[SectionEntry] {
        &self.entries
    }
}

pub(crate) fn parse_section(store: &(impl OneStore + ?Sized), filename: String) -> Result<Section> {
    let mut ctx = ParserContext {
        page: None,
        report: Report::new(),
        recognized_words: Default::default(),
    };

    let metadata = parse_metadata(store.data_root())?;
    let content = parse_content(store.data_root(), &mut ctx)?;

    let display_name = metadata
        .display_name
        .unwrap_or(filename)
        .trim_end_matches(".one")
        .to_string();

    let page_series = content
        .page_series
        .into_iter()
        .map(|page_series_id| parse_page_series(page_series_id, store, &mut ctx))
        .collect::<Result<_>>()?;

    Ok(Section {
        file_identity: FileIdentity::new(store.file_identity()),
        display_name,
        page_series,
        color: metadata.color,
        report: ctx.report,
    })
}

fn parse_content(
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<section_node::Data> {
    let content_root_id = space
        .content_root()
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("section has no content root".into()))?;
    let content_object = space.get_object(content_root_id).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("section content object is missing".into())
    })?;

    section_node::parse(content_object, ctx)
}

fn parse_metadata(space: &(impl ObjectSpace + ?Sized)) -> Result<section_metadata_node::Data> {
    let metadata_root_id = space
        .metadata_root()
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("section has no metadata root".into()))?;
    let metadata_object = space.get_object(metadata_root_id).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData("section metadata object is missing".into())
    })?;

    section_metadata_node::parse(metadata_object)
}
