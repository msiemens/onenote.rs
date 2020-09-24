use crate::one::property::object_reference::ObjectReference;
use crate::one::property::PropertyType;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) page: ExGuid,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::PageManifestNode.as_jcid());

    let page = ObjectReference::parse_vec(PropertyType::ContentChildNodes, object)
        .and_then(|ids| ids.first().copied())
        .expect("page manifest has no page");

    Data { page }
}
