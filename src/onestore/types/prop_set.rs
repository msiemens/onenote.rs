use crate::onestore::types::property::{PropertyId, PropertyValue};
use crate::Reader;
use indexmap::map::IndexMap;

#[derive(Debug)]
pub(crate) struct PropertySet(IndexMap<PropertyId, PropertyValue>);

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

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&PropertyId, &PropertyValue)> {
        self.0.iter()
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &PropertyValue> {
        self.0.values()
    }
}
