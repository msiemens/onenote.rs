use crate::errors::{ErrorKind, Result};
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::PropertyType;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) page: ExGuid,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::PageManifestNode.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let page = ObjectReference::parse_vec(PropertyType::ContentChildNodes, object)?
        .and_then(|ids| ids.first().copied())
        .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("page manifest has no page".into()))?;

    Ok(Data { page })
}
