use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Timestamp;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;

#[derive(Debug)]
pub(crate) struct Data {
    entity_guid: Guid,
    page_series: Vec<ExGuid>,
    created_at: Timestamp,
}

impl Data {
    pub(crate) fn page_series(&self) -> &[ExGuid] {
        &self.page_series
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::SectionNode.as_jcid());

    let entity_guid = simple::parse_guid(PropertyType::NotebookManagementEntityGuid, object)
        .expect("section has no guid");
    let page_series =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object).unwrap_or_default();
    let created_at = Timestamp::parse(PropertyType::TopologyCreationTimeStamp, object)
        .expect("section has no creation timestamp");

    Data {
        entity_guid,
        page_series,
        created_at,
    }
}
