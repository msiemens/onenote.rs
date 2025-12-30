use crate::errors::{ErrorKind, Result};
use crate::one::property::{PropertyType, simple};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::object::Object;

/// An ink data container.
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) embed_type: Option<u32>,
    pub(crate) source_url: String,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::IFrameNode)?;

    let embed_type = simple::parse_u32(PropertyType::ImageEmbedType, object)?;
    let source_url = simple::parse_string(PropertyType::ImageEmbeddedUrl, object)?
        .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("iframe has no source URL".into()))?;

    Ok(Data {
        embed_type,
        source_url,
    })
}
