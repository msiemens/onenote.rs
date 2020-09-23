use crate::one::property_set::page_series_node;
use crate::onenote::parser::page::{parse_page, Page};
use crate::onestore::object_space::ObjectSpace;
use crate::onestore::OneStore;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct PageSeries {
    pub pages: Vec<Page>,
}

pub(crate) fn parse_page_series(id: ExGuid, store: &OneStore) -> PageSeries {
    let object = store
        .data_root()
        .get_object(id)
        .expect("page series object is missing");
    let data = page_series_node::parse(object);
    let pages = data.page_spaces();

    let pages = pages
        .iter()
        .flat_map(|page_space_id| store.object_spaces().get(page_space_id))
        .filter(|page_space| !is_version_object_space(page_space))
        .map(|page_space| parse_page(page_space))
        .collect();

    PageSeries { pages }
}

fn is_version_object_space(space: &ObjectSpace) -> bool {
    let version_space_context =
        ExGuid::parse_str("7111497f-1b6b-4209-9491-c98b04cf4c5a", 1).unwrap();

    space.context() == version_space_context
}
