use crate::errors::{ErrorKind, Result};
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

pub(crate) fn parse_page_series(id: ExGuid, store: &OneStore) -> Result<PageSeries> {
    let object = store
        .data_root()
        .get_object(id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page series object is missing".into()))?;
    let data = page_series_node::parse(object)?;

    let pages = data
        .page_spaces
        .into_iter()
        .map(|page_space_id| {
            store
                .object_space(page_space_id)
                .ok_or_else(|| ErrorKind::MalformedOneNoteData("page space is missing".into()))
        })
        .map(|page_space| parse_page(page_space?))
        .collect::<Result<_>>()?;

    Ok(PageSeries { pages })
}
