use crate::one::property_set::page_series_node;
use crate::onenote::parser::page::{parse_page, Page};

use crate::onestore::OneStore;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct PageSeries {
    pages: Vec<Page>,
}

impl PageSeries {
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }
}

pub(crate) fn parse_page_series(id: ExGuid, store: &OneStore) -> PageSeries {
    let object = store
        .data_root()
        .get_object(id)
        .expect("page series object is missing");
    let data = page_series_node::parse(object);

    let pages = data
        .page_spaces
        .into_iter()
        .map(|page_space_id| {
            store
                .object_space(page_space_id)
                .expect("page space is missing")
        })
        .map(|page_space| parse_page(page_space))
        .collect();

    PageSeries { pages }
}
