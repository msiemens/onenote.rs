use crate::errors::{ErrorKind, Result};
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

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::TableRowNode.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)?;
    let cells = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)?
        .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("table row has no cells".into()))?;

    Ok(Data {
        last_modified,
        cells,
    })
}
