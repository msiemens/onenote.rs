use crate::one::property::object_reference::ObjectReference;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) children: Vec<ExGuid>,
    pub(crate) filename: Option<String>,
    pub(crate) ordering_id: Option<u32>,
    // FIXME: Color!?
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TocContainer.as_jcid());

    let children =
        ObjectReference::parse_vec(PropertyType::TocChildren, object).unwrap_or_default();
    let filename = simple::parse_string(PropertyType::FolderChildFilename, object)
        .map(|s| s.replace("^M", "+"))
        .map(|s| s.replace("^J", ","));
    let ordering_id = simple::parse_u32(PropertyType::NotebookElementOrderingID, object);

    Data {
        children,
        filename,
        ordering_id,
    }
}
