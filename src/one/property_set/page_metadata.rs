use crate::one::property::time::Timestamp;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::guid::Guid;

#[derive(Debug)]
pub(crate) struct Data {
    entity_guid: Guid,
    cached_title: String,
    schema_revision_in_order_to_read: Option<u32>, // FIXME: Force this?
    schema_revision_in_order_to_write: Option<u32>, // FIXME: Force this?
    page_level: i32,
    created_at: Timestamp,
    is_deleted: bool,
}

impl Data {
    pub(crate) fn page_level(&self) -> i32 {
        self.page_level
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::PageMetadata.as_jcid());

    let entity_guid = simple::parse_guid(PropertyType::NotebookManagementEntityGuid, object)
        .expect("page metadata has no guid");
    let cached_title = simple::parse_string(PropertyType::CachedTitleString, object)
        .expect("page metadata has no cached title");
    let schema_revision_in_order_to_read =
        simple::parse_u32(PropertyType::SchemaRevisionInOrderToRead, object);
    let schema_revision_in_order_to_write =
        simple::parse_u32(PropertyType::SchemaRevisionInOrderToWrite, object);
    let page_level = simple::parse_u32(PropertyType::PageLevel, object).unwrap_or(0) as i32;
    let created_at = Timestamp::parse(PropertyType::TopologyCreationTimeStamp, object)
        .expect("page metadata has no creation timestamp");
    let is_deleted =
        simple::parse_bool(PropertyType::IsDeletedGraphSpaceContent, object).unwrap_or_default();

    Data {
        entity_guid,
        cached_title,
        schema_revision_in_order_to_read,
        schema_revision_in_order_to_write,
        page_level,
        created_at,
        is_deleted,
    }
}
