use crate::errors::{ErrorKind, Result};
use crate::one::property_set::PropertySetId;
use crate::onenote::parser::embedded_file::{parse_embedded_file, EmbeddedFile};
use crate::onenote::parser::image::{parse_image, Image};
use crate::onenote::parser::outline::{parse_outline, Outline};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Clone, Debug)]
pub enum PageContent {
    Outline(Outline),
    Image(Image),
    EmbeddedFile(EmbeddedFile),
    Unknown,
}

impl PageContent {
    pub fn outline(&self) -> Option<&Outline> {
        if let PageContent::Outline(outline) = self {
            Some(outline)
        } else {
            None
        }
    }

    pub fn image(&self) -> Option<&Image> {
        if let PageContent::Image(image) = self {
            Some(image)
        } else {
            None
        }
    }

    pub fn embedded_file(&self) -> Option<&EmbeddedFile> {
        if let PageContent::EmbeddedFile(embedded_file) = self {
            Some(embedded_file)
        } else {
            None
        }
    }
}

pub(crate) fn parse_page_content(content_id: ExGuid, space: &ObjectSpace) -> Result<PageContent> {
    let content_type = space
        .get_object(content_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("page content is missing".into()))?
        .id();
    let id = PropertySetId::from_jcid(content_type).ok_or_else(|| {
        ErrorKind::MalformedOneNoteData(
            format!("invalid property set id: {:?}", content_type).into(),
        )
    })?;

    let content = match id {
        PropertySetId::ImageNode => PageContent::Image(parse_image(content_id, space)?),
        PropertySetId::EmbeddedFileNode => {
            PageContent::EmbeddedFile(parse_embedded_file(content_id, space)?)
        }
        PropertySetId::OutlineNode => PageContent::Outline(parse_outline(content_id, space)?),
        PropertySetId::InkNode => PageContent::Unknown,
        _ => {
            return Err(ErrorKind::MalformedOneNoteData(
                format!("invalid content type: {:?}", id).into(),
            )
            .into())
        }
    };

    Ok(content)
}
