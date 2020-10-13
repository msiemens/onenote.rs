use crate::errors::{ErrorKind, Result};
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Timestamp;
use crate::one::property::PropertyType;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) last_modified: Timestamp,
    pub(crate) author_most_recent: ExGuid,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::RevisionMetadata.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let last_modified =
        Timestamp::parse(PropertyType::LastModifiedTimeStamp, object)?.ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("revision has no last modified timestamp".into())
        })?;
    let author_most_recent = ObjectReference::parse(PropertyType::AuthorMostRecent, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("revision has no most recent author".into())
        })?;

    Ok(Data {
        last_modified,
        author_most_recent,
    })
}
