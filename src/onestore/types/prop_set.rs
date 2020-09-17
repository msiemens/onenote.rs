use crate::onestore::types::property::{PropertyId, PropertyValue};
use crate::Reader;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct PropertySet(HashMap<PropertyId, PropertyValue>);

impl PropertySet {
    pub(crate) fn parse(reader: Reader) -> PropertySet {
        let count = reader.get_u16_le();

        let property_ids: Vec<_> = (0..count).map(|_| PropertyId::parse(reader)).collect();

        let properties = property_ids
            .into_iter()
            .map(|id| (id, PropertyValue::parse(id, reader)))
            .collect();

        PropertySet(properties)
    }

    pub(crate) fn get(&self, id: PropertyId) -> Option<&PropertyValue> {
        self.0.get(&id)
    }
}
