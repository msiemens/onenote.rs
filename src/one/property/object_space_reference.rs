use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::property::PropertyValue;
use crate::types::exguid::ExGuid;

pub(crate) struct ObjectSpaceReference;

impl ObjectSpaceReference {
    pub(crate) fn parse_vec(prop_type: PropertyType, object: &Object) -> Option<Vec<ExGuid>> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value
                    .to_object_space_ids()
                    .expect("object space reference array is not a object space array")
            })
            .map(|count| {
                object
                    .props()
                    .object_space_ids()
                    .iter()
                    .skip(Self::get_offset(prop_type, object))
                    .take(count as usize)
                    .map(|id| Self::resolve_id(id, object))
                    .collect()
            })
    }

    fn get_offset(prop_type: PropertyType, object: &Object) -> usize {
        let prop_index = object
            .props()
            .properties()
            .iter()
            .position(|(id, _)| id.value() == prop_type as u32)
            .unwrap();

        object
            .props()
            .properties()
            .iter()
            .take(prop_index)
            .map(|(_, v)| match v {
                PropertyValue::ObjectSpaceId => 1,
                PropertyValue::ObjectSpaceIds(c) => *c as usize,
                _ => 0,
            })
            .sum()
    }

    fn resolve_id(id: &CompactId, object: &Object) -> ExGuid {
        object
            .mapping()
            .get_object(*id)
            .expect("id not defined in mapping")
    }
}
