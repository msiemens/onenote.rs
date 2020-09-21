use crate::one::property_set::PropertySetId;
use crate::onenote::parser::embedded_file::{parse_embedded_file, EmbeddedFile};
use crate::onenote::parser::image::{parse_image, Image};
use crate::onenote::parser::rich_text::{parse_rich_text, RichText};
use crate::onenote::parser::table::{parse_table, Table};
use crate::onestore::object_space::ObjectSpace;
use crate::onestore::revision::Revision;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub enum Content {
    RichText(RichText),
    Table(Table),
    Image(Image),
    EmbeddedFile(EmbeddedFile),
}

pub(crate) fn parse_content(content_id: ExGuid, rev: &Revision, space: &ObjectSpace) -> Content {
    let content_type = rev
        .resolve_object(content_id, space)
        .expect("page content is missing")
        .id();
    let id = PropertySetId::from_jcid(content_type).unwrap();

    match id {
        PropertySetId::ImageNode => Content::Image(parse_image(content_id, rev, space)),
        PropertySetId::EmbeddedFileNode => {
            Content::EmbeddedFile(parse_embedded_file(content_id, rev, space))
        }
        PropertySetId::RichTextNode => Content::RichText(parse_rich_text(content_id, rev, space)),
        PropertySetId::TableNode => Content::Table(parse_table(content_id, rev, space)),
        _ => panic!("invalid content type: {:?}", id),
    }
}
