use crate::errors::{ErrorKind, Result};
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::object_space_reference::ObjectSpaceReference;
use crate::one::property::time::Timestamp;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) entity_guid: Guid,
    pub(crate) page_spaces: Vec<CellId>,
    pub(crate) page_metadata: Vec<ExGuid>,
    pub(crate) created_at: Option<Timestamp>, // FIXME: Force this?
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::PageSeriesNode.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let entity_guid = simple::parse_guid(PropertyType::NotebookManagementEntityGuid, object)?
        .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("page series has no guid".into()))?;
    let page_spaces =
        ObjectSpaceReference::parse_vec(PropertyType::ChildGraphSpaceElementNodes, object)?
            .unwrap_or_default();
    let page_metadata =
        ObjectReference::parse_vec(PropertyType::MetaDataObjectsAboveGraphSpace, object)?
            .unwrap_or_default();
    let created_at = Timestamp::parse(PropertyType::TopologyCreationTimeStamp, object)?;

    let data = Data {
        entity_guid,
        page_spaces,
        page_metadata,
        created_at,
    };

    Ok(data)
}
