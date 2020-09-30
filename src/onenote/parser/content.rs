use crate::one::property_set::PropertySetId;
use crate::onenote::parser::embedded_file::{parse_embedded_file, EmbeddedFile};
use crate::onenote::parser::image::{parse_image, Image};
use crate::onenote::parser::rich_text::{parse_rich_text, RichText};
use crate::onenote::parser::table::{parse_table, Table};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub enum Content {
    RichText(RichText),
    Table(Table),
    Image(Image),
    EmbeddedFile(EmbeddedFile),
    Unknown,
}

impl Content {
    pub fn rich_text(&self) -> Option<&RichText> {
        if let Content::RichText(rich_text) = self {
            Some(rich_text)
        } else {
            None
        }
    }

    pub fn table(&self) -> Option<&Table> {
        if let Content::Table(table) = self {
            Some(table)
        } else {
            None
        }
    }

    pub fn image(&self) -> Option<&Image> {
        if let Content::Image(image) = self {
            Some(image)
        } else {
            None
        }
    }

    pub fn embedded_file(&self) -> Option<&EmbeddedFile> {
        if let Content::EmbeddedFile(file) = self {
            Some(file)
        } else {
            None
        }
    }
}

pub(crate) fn parse_content(content_id: ExGuid, space: &ObjectSpace) -> Content {
    let content_type = space
        .get_object(content_id)
        .expect("page content is missing")
        .id();
    let id = PropertySetId::from_jcid(content_type).unwrap();

    match id {
        PropertySetId::ImageNode => Content::Image(parse_image(content_id, space)),
        PropertySetId::EmbeddedFileNode => {
            Content::EmbeddedFile(parse_embedded_file(content_id, space))
        }
        PropertySetId::RichTextNode => Content::RichText(parse_rich_text(content_id, space)),
        PropertySetId::TableNode => Content::Table(parse_table(content_id, space)),
        PropertySetId::InkNode => Content::Unknown,
        _ => panic!("invalid content type: {:?}", id),
    }
}
