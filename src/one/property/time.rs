use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub struct Time(u32);

impl Time {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<Time> {
        object
            .props()
            .get(prop_type)
            .map(|value| Time(value.to_u32().expect("time value is not a u32")))
    }
}

#[derive(Debug)]
pub(crate) struct Timestamp(u64);

impl Timestamp {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Option<Timestamp> {
        object
            .props()
            .get(prop_type)
            .map(|value| Timestamp(value.to_u64().expect("timestamp value is not a u64")))
    }
}
