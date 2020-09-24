use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) last_modified: Time,
    pub(crate) children: Vec<ExGuid>,
    pub(crate) child_level: u8,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::OutlineGroup.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("outline group has no last modified time");
    let children =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object).unwrap_or_default();
    let child_level = simple::parse_u8(PropertyType::OutlineElementChildLevel, object)
        .expect("outline group has no child level");

    Data {
        last_modified,
        children,
        child_level,
    }
}
