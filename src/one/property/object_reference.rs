use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::references::References;
use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::property::PropertyValue;

pub(crate) struct ObjectReference;

impl ObjectReference {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<ExGuid>> {
        // Validate the value of the property
        let property = unwrap_or_return!(object.props().get(prop_type));
        property.to_object_id().ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("object reference is not a object id".into())
        })?;

        // Find the correct object reference
        let id = object
            .props()
            .object_ids()
            .iter()
            .nth(Self::get_offset(prop_type, object)?)
            .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("object id index corrupt".into()))?;

        Ok(Self::resolve_id(id, object))
    }

    pub(crate) fn parse_vec(
        prop_type: PropertyType,
        object: &Object,
    ) -> Result<Option<Vec<ExGuid>>> {
        let prop = unwrap_or_return!(object.props().get(prop_type));
        let count = prop.to_object_ids().ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData(
                "object reference array is not a object id array".into(),
            )
        })?;
        let object_refs = object.props().object_ids();
        let object_ids = object_refs
            .iter()
            .skip(Self::get_offset(prop_type, object)?)
            .take(count as usize)
            .flat_map(|id| Self::resolve_id(id, object))
            .collect();

        Ok(Some(object_ids))
    }

    pub(crate) fn get_offset(prop_type: PropertyType, object: &Object) -> Result<usize> {
        let predecessors = References::get_predecessors(prop_type, object)?;
        let offset = Self::count_references(predecessors);

        Ok(offset)
    }

    pub(crate) fn count_references<'a>(props: impl Iterator<Item = &'a PropertyValue>) -> usize {
        props
            .map(|v| match v {
                PropertyValue::ObjectId => 1,
                PropertyValue::ObjectIds(c) => *c as usize,
                PropertyValue::PropertyValues(_, sets) => sets
                    .iter()
                    .map(|set| Self::count_references(set.values()))
                    .sum(),
                _ => 0,
            })
            .sum()
    }

    fn resolve_id(id: &CompactId, object: &Object) -> Option<ExGuid> {
        object.mapping().get_object(*id)
    }
}
