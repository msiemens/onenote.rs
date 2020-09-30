use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::PropertyType;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) last_modified: Option<Time>,
    pub(crate) cells: Vec<ExGuid>,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TableRowNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object);
    let cells = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)
        .expect("table row has no cells");

    Data {
        last_modified,
        cells,
    }
}
