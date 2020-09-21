use crate::one::property_set::PropertySetId;
use crate::onenote::parser::embedded_file::{parse_embedded_file, EmbeddedFile};
use crate::onenote::parser::image::{parse_image, Image};
use crate::onenote::parser::outline::{parse_outline, Outline};
use crate::onestore::revision::Revision;
use crate::onestore::OneStore;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub enum PageContent {
    Outline(Outline),
    Image(Image),
    EmbeddedFile(EmbeddedFile),
}

pub(crate) fn parse_page_content(
    content_id: ExGuid,
    rev: &Revision,
    store: &OneStore,
) -> PageContent {
    let content_type = rev
        .resolve_object(content_id, store)
        .expect("page content is missing")
        .id();
    let id = PropertySetId::from_jcid(content_type)
        .unwrap_or_else(|| panic!("invalid property set id: {:?}", content_type));

    match id {
        PropertySetId::ImageNode => PageContent::Image(parse_image(content_id, rev, store)),
        PropertySetId::EmbeddedFileNode => {
            PageContent::EmbeddedFile(parse_embedded_file(content_id, rev, store))
        }
        PropertySetId::OutlineNode => PageContent::Outline(parse_outline(content_id, rev, store)),
        _ => panic!("invalid content type: {:?}", id),
    }
}
