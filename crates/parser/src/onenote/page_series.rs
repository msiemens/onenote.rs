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

        // One page that fails to parse (seen: "page manifest has no page" in a section
        // OneNote for Mac had only partly downloaded) must not fail every other page of
        // its section. Skip it with a warning instead.
        match parse_page(page_space, ctx) {
            Ok(page) => pages.push(page),
            Err(e) => {
                warn!(
                    ctx,
                    "page {:?} could not be parsed, skipping it: {}", page_space_id, e
                );
                // `parse_page` clears these only when it succeeds. Left set, the failed page
                // would own the next page's early warnings and lend it recognized words.
                ctx.page = None;
                ctx.recognized_words.clear();
            }
        }
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

    /// A store that answers the first page space lookup with the data root, which is a
    /// real object space but not a page, so that page fails to parse.
    #[derive(Debug)]
    struct UnparseableFirstPage<S> {
        inner: S,
        broken: Cell<Option<CellId>>,
    }

    impl<S: OneStore> OneStore for UnparseableFirstPage<S> {
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
            if self.broken.get().is_none_or(|broken| broken == id) {
                self.broken.set(Some(id));
                return Some(self.inner.data_root());
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

    #[test]
    fn skips_a_page_that_fails_to_parse() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/samples/joplin/aaa/Quick Notes.one"
        );
        let data = std::fs::read(path).unwrap();
        let packaging = OneStorePackaging::parse(&mut Reader::new(&data)).unwrap();

        let (pages, warnings) = page_count(&parse_store(&packaging).unwrap());

        let store = UnparseableFirstPage {
            inner: parse_store(&packaging).unwrap(),
            broken: Cell::new(None),
        };
        let (pages_with_broken_page, warnings_with_broken_page) = page_count(&store);

        assert!(store.broken.get().is_some());
        assert_eq!(pages_with_broken_page, pages - 1);
        assert_eq!(warnings_with_broken_page, warnings + 1);
    }
}

#[cfg(test)]
mod failed_page_state_tests {
    use crate::errors::Result;
    use crate::fs::FileSource;
    use crate::fs::file_source::BytesSource;
    use crate::fsshttpb::data::cell_id::CellId;
    use crate::fsshttpb::data::exguid::ExGuid;
    use crate::one::property_set::{page_manifest_node, page_node, page_series_node, section_node};
    use crate::onenote::ParserContext;
    use crate::onenote::page_series::parse_page_series;
    use crate::onenote::parse_store_auto;
    use crate::onestore::{Object, ObjectSpace, OneStore, OneStoreType};
    use crate::shared::guid::Guid;
    use crate::warn::Report;
    use bytes::Bytes;
    use std::cell::Cell;
    use std::sync::Arc;

    /// A page space whose first content object is missing, so the page fails in
    /// `parse_page_content`: after `ctx.page` is set and its recognition is parsed.
    #[derive(Debug)]
    struct ContentHidden<'a> {
        inner: &'a dyn ObjectSpace,
        hidden: ExGuid,
    }

    impl ObjectSpace for ContentHidden<'_> {
        fn get_object(&self, id: ExGuid) -> Option<&Object> {
            if id == self.hidden {
                return None;
            }
            self.inner.get_object(id)
        }

        fn content_root(&self) -> Option<ExGuid> {
            self.inner.content_root()
        }

        fn metadata_root(&self) -> Option<ExGuid> {
            self.inner.metadata_root()
        }
    }

    /// Serves `broken` for `target`, and hides every page space looked up after it, so the
    /// broken page is the last one parsed and nothing afterwards resets the context.
    #[derive(Debug)]
    struct BreakOnePage<'a, S: ?Sized> {
        inner: &'a S,
        target: CellId,
        broken: ContentHidden<'a>,
        served: Cell<bool>,
    }

    impl<S: OneStore + ?Sized> OneStore for BreakOnePage<'_, S> {
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
            if id == self.target {
                self.served.set(true);
                return Some(&self.broken);
            }
            if self.served.get() {
                return None;
            }
            self.inner.object_space(id)
        }
    }

    fn new_ctx() -> ParserContext {
        ParserContext {
            page: None,
            report: Report::new(),
            recognized_words: Default::default(),
        }
    }

    fn page_data(space: &dyn ObjectSpace) -> page_node::Data {
        let manifest_object = space.get_object(space.content_root().unwrap()).unwrap();
        let manifest = page_manifest_node::parse(manifest_object).unwrap();
        page_node::parse(space.get_object(manifest.page).unwrap()).unwrap()
    }

    fn page_series_ids(store: &(impl OneStore + ?Sized), ctx: &mut ParserContext) -> Vec<ExGuid> {
        let root = store.data_root();
        let content = root.get_object(root.content_root().unwrap()).unwrap();
        section_node::parse(content, ctx).unwrap().page_series
    }

    #[test]
    fn a_page_that_fails_midway_leaves_no_page_state_behind() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/samples/handwriting_recognition.one"
        );
        let data = std::fs::read(path).unwrap();
        // A desktop-format (MS-ONESTORE) sample: the only one with recognized handwriting.
        let source: Arc<dyn FileSource> = Arc::new(BytesSource::new(Bytes::copy_from_slice(&data)));
        let parsed = parse_store_auto(source).unwrap();
        let inner = parsed.as_onestore();

        let mut ctx = new_ctx();
        let (target, space) = page_series_ids(inner, &mut ctx)
            .into_iter()
            .flat_map(|id| {
                page_series_node::parse(inner.data_root().get_object(id).unwrap())
                    .unwrap()
                    .page_spaces
            })
            .filter_map(|cell| inner.object_space(cell).map(|space| (cell, space)))
            .find(|(_, space)| {
                let data = page_data(*space);
                data.recognized_text.is_some() && !data.content.is_empty()
            })
            .expect("the sample has a page with recognized handwriting");

        let store = BreakOnePage {
            inner,
            target,
            broken: ContentHidden {
                inner: space,
                hidden: page_data(space).content[0],
            },
            served: Cell::new(false),
        };

        let mut ctx = new_ctx();
        for id in page_series_ids(&store, &mut ctx) {
            parse_page_series(id, &store, &mut ctx).unwrap();
        }

        let skipped: Vec<_> = ctx
            .report
            .warnings()
            .iter()
            .filter(|w| w.message().contains("could not be parsed"))
            .collect();
        assert_eq!(skipped.len(), 1);
        // The skip warning names the page that failed: it was already identified.
        assert!(skipped[0].page().is_some());

        assert_eq!(ctx.page, None, "the failed page is still the current page");
        assert!(
            ctx.recognized_words.is_empty(),
            "{} recognized words of the failed page were left behind",
            ctx.recognized_words.len()
        );
    }
}
