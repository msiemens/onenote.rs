use crate::one::property_set::PropertySetId;
use crate::onenote::parser::embedded_file::{parse_embedded_file, EmbeddedFile};
use crate::onenote::parser::image::{parse_image, Image};
use crate::onenote::parser::outline::{parse_outline, Outline};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub enum PageContent {
    Outline(Outline),
    Image(Image),
    EmbeddedFile(EmbeddedFile),
    Unknown,
}

pub(crate) fn parse_page_content(content_id: ExGuid, space: &ObjectSpace) -> PageContent {
    let content_type = space
        .get_object(content_id)
        .expect("page content is missing")
        .id();
    let id = PropertySetId::from_jcid(content_type)
        .unwrap_or_else(|| panic!("invalid property set id: {:?}", content_type));

    match id {
        PropertySetId::ImageNode => PageContent::Image(parse_image(content_id, space)),
        PropertySetId::EmbeddedFileNode => {
            PageContent::EmbeddedFile(parse_embedded_file(content_id, space))
        }
        PropertySetId::OutlineNode => PageContent::Outline(parse_outline(content_id, space)),
        PropertySetId::InkNode => PageContent::Unknown,
        _ => panic!("invalid content type: {:?}", id),
    }
}
