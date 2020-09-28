use crate::one::property_set::{section_metadata_node, section_node};
use crate::onenote::parser::page_series::{parse_page_series, PageSeries};
use crate::onestore::object_space::ObjectSpace;
use crate::onestore::OneStore;

#[derive(Debug)]
pub struct Section {
    pub display_name: String,
    pub page_series: Vec<PageSeries>,
}

pub(crate) fn parse_section(store: OneStore, filename: String) -> Section {
    let metadata = parse_metadata(store.data_root());
    let content = parse_content(store.data_root());

    let display_name = metadata.display_name.unwrap_or_else(|| {
        filename
            .strip_suffix(".one")
            .unwrap_or(&*filename)
            .to_string()
    });

    let page_series = content
        .page_series
        .into_iter()
        .map(|page_series_id| parse_page_series(page_series_id, &store))
        .collect();

    Section {
        display_name,
        page_series,
    }
}

fn parse_content(space: &ObjectSpace) -> section_node::Data {
    let content_root_id = space.content_root().expect("section has no content root");
    let content_object = space
        .get_object(content_root_id)
        .expect("section content object is missing");

    section_node::parse(content_object)
}

fn parse_metadata(space: &ObjectSpace) -> section_metadata_node::Data {
    let metadata_root_id = space.metadata_root().expect("section has no metadata root");
    let metadata_object = space
        .get_object(metadata_root_id)
        .expect("section metadata object is missing");

    section_metadata_node::parse(metadata_object)
}
