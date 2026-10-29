use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property_set::page_series_node;
use crate::onenote::ParserContext;
use crate::onenote::page::{Page, parse_page};
use crate::onestore::OneStore;

/// A series of page.
///
/// See [\[MS-ONE\] 1.3.2] and [\[MS-ONE\] 2.2.18].
///
/// [\[MS-ONE\] 1.3.2]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/2dd687ac-f36b-4723-b959-4d60c8a90ca9
/// [\[MS-ONE\] 2.2.18]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/e2957d3b-a2a8-4756-8662-4e67fefa9f4e
#[derive(Clone, Debug)]
pub struct PageSeries {
    pages: Vec<Page>,
}

impl PageSeries {
    /// The pages contained in this page series.
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }
}

pub(crate) fn parse_page_series(
    id: ExGuid,
    store: &(impl OneStore + ?Sized),
    ctx: &mut ParserContext,
) -> Result<PageSeries> {
    let object = store
        .data_root()
        .get_object(id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page series object is missing".into()))?;
    let data = page_series_node::parse(object)?;

    let mut pages = Vec::new();

    for page_space_id in data.page_spaces {
        // A section that was downloaded while it was being edited can reference a page space
        // that is not in the file. Skip that page instead of failing the whole section.
        let Some(page_space) = store.object_space(page_space_id) else {
            warn!(
                ctx,
                "page space {:?} is missing, skipping page", page_space_id
            );
            continue;
        };

        pages.push(parse_page(page_space, ctx)?);
    }

    Ok(PageSeries { pages })
}

#[cfg(test)]
mod tests {
    use crate::errors::Result;
    use crate::fsshttpb::data::cell_id::CellId;
    use crate::fsshttpb::packaging::OneStorePackaging;
    use crate::onenote::section::parse_section;
    use crate::onestore::fsshttpb::parse_store;
    use crate::onestore::{ObjectSpace, OneStore, OneStoreType};
    use crate::reader::Reader;
    use crate::shared::guid::Guid;
    use std::cell::Cell;

    /// A store that hides the first page space that is looked up.
    #[derive(Debug)]
    struct MissingFirstSpace<S> {
        inner: S,
        missing: Cell<Option<CellId>>,
    }

    impl<S: OneStore> OneStore for MissingFirstSpace<S> {
        fn file_identity(&self) -> Guid {
            self.inner.file_identity()
        }

        fn get_type(&self) -> Result<OneStoreType> {
            self.inner.get_type()
        }

        fn data_root(&self) -> &dyn ObjectSpace {
            self.inner.data_root()
        }

        fn object_space(&self, id: CellId) -> Option<&dyn ObjectSpace> {
            if self.missing.get().is_none_or(|missing| missing == id) {
                self.missing.set(Some(id));
                return None;
            }

            self.inner.object_space(id)
        }
    }

    fn page_count(store: &(impl OneStore + ?Sized)) -> (usize, usize) {
        let section = parse_section(store, "Quick Notes.one".into()).unwrap();
        let pages = section
            .page_series()
            .iter()
            .map(|series| series.pages().len())
            .sum();

        (pages, section.report().warnings().len())
    }

    #[test]
    fn skips_a_page_whose_page_space_is_missing() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/samples/joplin/aaa/Quick Notes.one"
        );
        let data = std::fs::read(path).unwrap();
        let packaging = OneStorePackaging::parse(&mut Reader::new(&data)).unwrap();

        let (pages, warnings) = page_count(&parse_store(&packaging).unwrap());

        let store = MissingFirstSpace {
            inner: parse_store(&packaging).unwrap(),
            missing: Cell::new(None),
        };
        let (pages_with_missing_space, warnings_with_missing_space) = page_count(&store);

        assert!(store.missing.get().is_some());
        assert_eq!(pages_with_missing_space, pages - 1);
        assert_eq!(warnings_with_missing_space, warnings + 1);
    }
}
