use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{page_manifest_node, page_metadata, page_node, title_node};
use crate::onenote::parser::outline::{parse_outline, Outline};
use crate::onenote::parser::page_content::{parse_page_content, PageContent};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Page {
    pub(crate) title: Option<Title>,
    pub(crate) level: i32,
    pub(crate) author: Option<String>,
    pub(crate) height: Option<f32>,
    pub(crate) contents: Vec<PageContent>,
}

#[derive(Debug)]
pub struct Title {
    pub(crate) contents: Outline,
    pub(crate) offset_horizontal: f32,
    pub(crate) offset_vertical: f32,
    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,
}

pub(crate) fn parse_page(page_space: &ObjectSpace) -> Page {
    let metadata = parse_metadata(page_space);
    let manifest = parse_manifest(page_space);

    let data = parse_data(manifest, page_space);

    let title = data.title().map(|id| parse_title(id, page_space));
    let level = metadata.page_level();

    let contents = data
        .content()
        .iter()
        .map(|id| parse_page_content(*id, page_space))
        .collect();

    Page {
        title,
        level,
        author: data.author().map(|author| author.name().to_string()),
        height: data.page_height(),
        contents,
    }
}

fn parse_title(title_id: ExGuid, space: &ObjectSpace) -> Title {
    let title_object = space.get_object(title_id).expect("title object is missing");
    let title = title_node::parse(title_object);
    let outline_id = title
        .children()
        .first()
        .copied()
        .expect("title has no outline");

    let contents = parse_outline(outline_id, space);

    Title {
        contents,
        offset_horizontal: title.offset_horizontal(),
        offset_vertical: title.offset_vertical(),
        layout_alignment_in_parent: title.layout_alignment_in_parent(),
        layout_alignment_self: title.layout_alignment_self(),
    }
}

fn parse_data(manifest: page_manifest_node::Data, space: &ObjectSpace) -> page_node::Data {
    let page_id = manifest.page();
    let page_object = space.get_object(page_id).expect("page object is missing");

    page_node::parse(page_object)
}

fn parse_manifest(space: &ObjectSpace) -> page_manifest_node::Data {
    let page_manifest_id = space.content_root().expect("page content id is missing");
    let page_manifest_object = space
        .get_object(page_manifest_id)
        .expect("page object is missing");

    page_manifest_node::parse(page_manifest_object)
}

fn parse_metadata(space: &ObjectSpace) -> page_metadata::Data {
    let metadata_id = space.metadata_root().expect("page metadata id is missing");
    let metadata_object = space
        .get_object(metadata_id)
        .expect("page metadata object is missing");

    page_metadata::parse(metadata_object)
}
