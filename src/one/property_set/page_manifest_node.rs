use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::PropertyType;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::object::Object;

/// A page manifest.
///
/// See [\[MS-ONE\] 2.2.34].
///
/// [\[MS-ONE\] 2.2.34]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/284dd0c5-786f-499f-8ca3-454f85091b29
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) page: ExGuid,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::PageManifestNode)?;

    let page = ObjectReference::parse_vec(PropertyType::ContentChildNodes, object)?
        .and_then(|ids| ids.first().copied())
        .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("page manifest has no page".into()))?;

    Ok(Data { page })
}
