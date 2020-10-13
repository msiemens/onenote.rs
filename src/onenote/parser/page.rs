use crate::errors::{ErrorKind, Result};
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{page_manifest_node, page_metadata, page_node, title_node};
use crate::onenote::parser::outline::{parse_outline, Outline};
use crate::onenote::parser::page_content::{parse_page_content, PageContent};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Page {
    title: Option<Title>,
    level: i32,
    author: Option<String>,
    height: Option<f32>,
    contents: Vec<PageContent>,
}

impl Page {
    pub fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    pub fn level(&self) -> i32 {
        self.level
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    pub fn height(&self) -> Option<f32> {
        self.height
    }

    pub fn contents(&self) -> &[PageContent] {
        &self.contents
    }

    pub fn title_text(&self) -> Option<&str> {
        self.title
            .as_ref()
            .and_then(|title| title.contents.first())
            .and_then(Self::outline_text)
            .or_else(|| {
                self.contents
                    .iter()
                    .filter_map(|page_content| page_content.outline())
                    .filter_map(Self::outline_text)
                    .next()
            })
    }

    fn outline_text(outline: &Outline) -> Option<&str> {
        outline
            .items
            .first()
            .and_then(|outline_item| outline_item.element())
            .and_then(|outline_element| outline_element.contents.first())
            .and_then(|content| content.rich_text())
            .and_then(|text| Some(&*text.text).filter(|s| !s.is_empty()))
    }
}

#[derive(Debug)]
pub struct Title {
    pub(crate) contents: Vec<Outline>,
    pub(crate) offset_horizontal: f32,
    pub(crate) offset_vertical: f32,
    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,
}

impl Title {
    pub fn contents(&self) -> &[Outline] {
        &self.contents
    }

    pub fn offset_horizontal(&self) -> f32 {
        self.offset_horizontal
    }

    pub fn offset_vertical(&self) -> f32 {
        self.offset_vertical
    }

    pub fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }
}

pub(crate) fn parse_page(page_space: &ObjectSpace) -> Result<Page> {
    let metadata = parse_metadata(page_space)?;
    let manifest = parse_manifest(page_space)?;

    let data = parse_data(manifest, page_space)?;

    let title = data
        .title
        .map(|id| parse_title(id, page_space))
        .transpose()?;
    let level = metadata.page_level;

    let contents = data
        .content
        .into_iter()
        .map(|content_id| parse_page_content(content_id, page_space))
        .collect::<Result<_>>()?;

    Ok(Page {
        title,
        level,
        author: data.author.map(|author| author.into_value()),
        height: data.page_height,
        contents,
    })
}

fn parse_title(title_id: ExGuid, space: &ObjectSpace) -> Result<Title> {
    let title_object = space
        .get_object(title_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("title object is missing".into()))?;
    let title = title_node::parse(title_object)?;
    let contents = title
        .children
        .into_iter()
        .map(|outline_id| parse_outline(outline_id, space))
        .collect::<Result<_>>()?;

    Ok(Title {
        contents,
        offset_horizontal: title.offset_horizontal,
        offset_vertical: title.offset_vertical,
        layout_alignment_in_parent: title.layout_alignment_in_parent,
        layout_alignment_self: title.layout_alignment_self,
    })
}

fn parse_data(manifest: page_manifest_node::Data, space: &ObjectSpace) -> Result<page_node::Data> {
    let page_id = manifest.page;
    let page_object = space
        .get_object(page_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page object is missing".into()))?;

    page_node::parse(page_object)
}

fn parse_manifest(space: &ObjectSpace) -> Result<page_manifest_node::Data> {
    let page_manifest_id = space
        .content_root()
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page content id is missing".into()))?;
    let page_manifest_object = space
        .get_object(page_manifest_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page object is missing".into()))?;

    page_manifest_node::parse(page_manifest_object)
}

fn parse_metadata(space: &ObjectSpace) -> Result<page_metadata::Data> {
    let metadata_id = space
        .metadata_root()
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page metadata id is missing".into()))?;
    let metadata_object = space
        .get_object(metadata_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page metadata object is missing".into()))?;

    page_metadata::parse(metadata_object)
}
