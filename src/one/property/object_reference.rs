use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::property::PropertyValue;
use crate::types::exguid::ExGuid;

pub(crate) struct ObjectReference;

impl ObjectReference {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<ExGuid> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value
                    .to_object_id()
                    .expect("object reference is not a object id")
            })
            .map(|_| {
                object
                    .props()
                    .object_ids()
                    .iter()
                    .nth(Self::get_offset(prop_type, object))
                    .expect("object id index corrupt")
            })
            .map(|id| Self::resolve_id(id, object))
    }

    pub(crate) fn parse_vec(prop_type: PropertyType, object: &Object) -> Option<Vec<ExGuid>> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value
                    .to_object_ids()
                    .expect("object reference array is not a object id array")
            })
            .map(|count| {
                object
                    .props()
                    .object_ids()
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
                PropertyValue::ObjectId => 1,
                PropertyValue::ObjectIds(c) => *c as usize,
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
