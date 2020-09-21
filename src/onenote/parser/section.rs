use crate::one::property_set::{section_metadata_node, section_node};
use crate::onenote::parser::page_series::{parse_page_series, PageSeries};
use crate::onestore::object_space::ObjectSpace;
use crate::onestore::revision::Revision;
use crate::onestore::OneStore;

#[derive(Debug)]
pub struct Section {
    pub(crate) display_name: Option<String>,
    pub(crate) page_series: Vec<PageSeries>,
}

pub(crate) fn parse_section(rev: &Revision, space: &ObjectSpace, store: &OneStore) -> Section {
    let metadata = parse_metadata(rev, space);
    let content = parse_content(rev, space);

    let display_name = metadata.display_name().map(String::from);

    let page_series = content
        .page_series()
        .iter()
        .map(|page_series_id| parse_page_series(*page_series_id, rev, space, store))
        .collect();

    Section {
        display_name,
        page_series,
    }
}

fn parse_content(rev: &Revision, space: &ObjectSpace) -> section_node::Data {
    let content_root_id = rev.content_root().expect("section has no content root");
    let content_object = rev
        .resolve_object(content_root_id, space)
        .expect("section content object is missing");

    section_node::parse(content_object)
}

fn parse_metadata(rev: &Revision, space: &ObjectSpace) -> section_metadata_node::Data {
    let metadata_root_id = rev.metadata_root().expect("section has no metadata root");
    let metadata_object = rev
        .resolve_object(metadata_root_id, space)
        .expect("section metadata object is missing");

    section_metadata_node::parse(metadata_object)
}
