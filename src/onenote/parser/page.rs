use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{page_manifest_node, page_metadata, page_node, title_node};
use crate::onenote::parser::outline::{parse_outline, Outline};
use crate::onenote::parser::page_content::{parse_page_content, PageContent};
use crate::onestore::revision::Revision;
use crate::onestore::OneStore;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Page {
    pub(crate) title: Title,
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

pub(crate) fn parse_page(page_space_id: ExGuid, store: &OneStore) -> Page {
    let page_space = store
        .object_spaces()
        .get(&page_space_id)
        .expect("page space is missing");
    let rev = page_space
        .find_root_revision()
        .expect("no page space root revision");

    let metadata = parse_metadata(store, rev);

    let manifest = parse_manifest(store, rev);
    let data = parse_data(manifest, store, rev);

    let title = parse_title(data.title(), rev, store);
    let level = metadata.page_level();

    // FIXME: Correctly parse title (see below)

    let contents = data
        .content()
        .iter()
        .map(|id| parse_page_content(*id, &rev, store))
        .collect();

    Page {
        title,
        level,
        author: data.author().map(|author| author.name().to_string()),
        height: data.page_height(),
        contents,
    }
}

fn parse_title(title_id: ExGuid, rev: &Revision, store: &OneStore) -> Title {
    let title_object = rev
        .resolve_object(title_id, store)
        .expect("title object is missing");
    let title = title_node::parse(title_object);
    let outline_id = title
        .children()
        .first()
        .copied()
        .expect("title has no outline");

    let contents = parse_outline(outline_id, rev, store);

    Title {
        contents,
        offset_horizontal: title.offset_horizontal(),
        offset_vertical: title.offset_vertical(),
        layout_alignment_in_parent: title.layout_alignment_in_parent(),
        layout_alignment_self: title.layout_alignment_self(),
    }
}

fn parse_data(
    manifest: page_manifest_node::Data,
    store: &OneStore,
    rev: &Revision,
) -> page_node::Data {
    let page_id = manifest.page();
    let page_object = rev
        .resolve_object(page_id, store)
        .expect("page object is missing");

    page_node::parse(page_object)
}

fn parse_manifest(store: &OneStore, rev: &Revision) -> page_manifest_node::Data {
    let page_manifest_id = rev.content_root().expect("page content id is missing");
    let page_manifest_object = rev
        .resolve_object(page_manifest_id, store)
        .expect("page object is missing");

    page_manifest_node::parse(page_manifest_object)
}

fn parse_metadata(store: &OneStore, rev: &Revision) -> page_metadata::Data {
    let metadata_id = rev.metadata_root().expect("page metadata id is missing");
    let metadata_object = rev
        .resolve_object(metadata_id, store)
        .expect("page metadata object is missing");

    page_metadata::parse(metadata_object)
}
