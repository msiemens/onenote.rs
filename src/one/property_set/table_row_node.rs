use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::PropertyType;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified: Time,
    cells: Vec<ExGuid>,
}

impl Data {
    pub(crate) fn cells(&self) -> &[ExGuid] {
        &self.cells
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TableRowNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("table row has no last modified time");
    let cells = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)
        .expect("table row has no cells");

    Data {
        last_modified,
        cells,
    }
}
