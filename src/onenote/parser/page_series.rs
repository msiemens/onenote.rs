use crate::one::property_set::page_series_node;
use crate::onenote::parser::page::{parse_page, Page};
use crate::onestore::revision::Revision;
use crate::onestore::OneStore;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct PageSeries {
    pub(crate) pages: Vec<Page>,
}

pub(crate) fn parse_page_series(id: ExGuid, rev: &Revision, store: &OneStore) -> PageSeries {
    let object = rev
        .resolve_object(id, store)
        .expect("page series object is missing");
    let data = page_series_node::parse(object);
    let pages = data.page_spaces();

    let pages = pages
        .iter()
        .filter(|space_id| !is_version_object_space(**space_id, store).expect("space is missig"))
        .map(|page_space_id| parse_page(*page_space_id, store))
        .collect();

    PageSeries { pages }
}

fn is_version_object_space(space_id: ExGuid, store: &OneStore) -> Option<bool> {
    let space = store.object_spaces().get(&space_id)?;
    let version_space_context =
        ExGuid::parse_str("7111497f-1b6b-4209-9491-c98b04cf4c5a", 1).unwrap();

    Some(space.context() == version_space_context)
}
