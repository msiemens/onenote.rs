use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Timestamp;
use crate::one::property::PropertyType;

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified: Timestamp,
    author_most_recent: ExGuid,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::RevisionMetadata.as_jcid());

    let last_modified = Timestamp::parse(PropertyType::LastModifiedTimeStamp, object)
        .expect("revision has no last modified timestamp");
    let author_most_recent = ObjectReference::parse(PropertyType::AuthorMostRecent, object)
        .expect("revision has no most recent author");

    Data {
        last_modified,
        author_most_recent,
    }
}
