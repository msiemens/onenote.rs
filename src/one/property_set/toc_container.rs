use crate::one::property::object_reference::ObjectReference;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    children: Vec<ExGuid>,
    filename: Option<String>,
    // FIXME: Color!?
}

impl Data {
    pub(crate) fn children(&self) -> &Vec<ExGuid> {
        &self.children
    }
    
    pub(crate) fn filename(&self) -> &Option<String> {
        &self.filename
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TocContainer.as_jcid());

    let children =
        ObjectReference::parse_vec(PropertyType::TocChildren, object).unwrap_or_default();
    let filename = simple::parse_string(PropertyType::SectionFileName, object);

    Data { children, filename }
}
