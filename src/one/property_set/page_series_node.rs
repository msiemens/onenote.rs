use crate::one::property::object_reference::ObjectReference;
use crate::one::property::object_space_reference::ObjectSpaceReference;
use crate::one::property::time::Timestamp;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;

#[derive(Debug)]
pub(crate) struct Data {
    entity_guid: Guid,
    page_spaces: Vec<ExGuid>,
    page_metadata: Vec<ExGuid>,
    created_at: Option<Timestamp>, // FIXME: Force this?
}

impl Data {
    pub(crate) fn page_spaces(&self) -> &[ExGuid] {
        &self.page_spaces
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::PageSeriesNode.as_jcid());

    let entity_guid = simple::parse_guid(PropertyType::NotebookManagementEntityGuid, object)
        .expect("page series has no guid");
    let page_spaces =
        ObjectSpaceReference::parse_vec(PropertyType::ChildGraphSpaceElementNodes, object)
            .unwrap_or_default();
    let page_metadata =
        ObjectReference::parse_vec(PropertyType::MetaDataObjectsAboveGraphSpace, object)
            .unwrap_or_default();
    let created_at = Timestamp::parse(PropertyType::TopologyCreationTimeStamp, object);

    Data {
        entity_guid,
        page_spaces,
        page_metadata,
        created_at,
    }
}