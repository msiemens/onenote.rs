use crate::errors::{ErrorKind, Result};
use crate::one::property::references::References;
use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::property::PropertyValue;
use crate::types::cell_id::CellId;

pub(crate) struct ObjectSpaceReference;

impl ObjectSpaceReference {
    pub(crate) fn parse_vec(
        prop_type: PropertyType,
        object: &Object,
    ) -> Result<Option<Vec<CellId>>> {
        let prop = unwrap_or_return!(object.props().get(prop_type));
        let count = prop.to_object_space_ids().ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData(
                "object space reference array is not a object id array".into(),
            )
        })?;
        let object_refs = object.props().object_space_ids();
        let object_space_ids = object_refs
            .iter()
            .skip(Self::get_offset(prop_type, object)?)
            .take(count as usize)
            .flat_map(|id| Self::resolve_id(id, object))
            .collect();

        Ok(Some(object_space_ids))
    }

    pub(crate) fn get_offset(prop_type: PropertyType, object: &Object) -> Result<usize> {
        let predecessors = References::get_predecessors(prop_type, object)?;
        let offset = Self::count_references(predecessors);

        Ok(offset)
    }

    pub(crate) fn count_references<'a>(props: impl Iterator<Item = &'a PropertyValue>) -> usize {
        props
            .map(|v| match v {
                PropertyValue::ObjectSpaceId => 1,
                PropertyValue::ObjectSpaceIds(c) => *c as usize,
                PropertyValue::PropertyValues(_, sets) => sets
                    .iter()
                    .map(|set| Self::count_references(set.values()))
                    .sum(),
                _ => 0,
            })
            .sum()
    }

    fn resolve_id(id: &CompactId, object: &Object) -> Result<CellId> {
        object
            .mapping()
            .get_object_space(*id)
            .ok_or_else(|| ErrorKind::MalformedOneNoteFileData("id not defined in mapping".into()))
            .map_err(|e| e.into())
    }
}
