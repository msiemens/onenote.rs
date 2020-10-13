use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub struct Time(u32);

impl Time {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<Time>> {
        let time = object
            .props()
            .get(prop_type)
            .map(|value| {
                value.to_u32().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("time value is not a u32".into())
                })
            })
            .transpose()?
            .map(Time);

        Ok(time)
    }
}

#[derive(Debug)]
pub(crate) struct Timestamp(u64);

impl Timestamp {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Option<Timestamp>> {
        let timestamp = object
            .props()
            .get(prop_type)
            .map(|value| {
                value.to_u64().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("timestamp value is not a u64".into())
                })
            })
            .transpose()?
            .map(Timestamp);

        Ok(timestamp)
    }
}
