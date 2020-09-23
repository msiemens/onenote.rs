use crate::onestore::types::property::{PropertyId, PropertyValue};
use crate::Reader;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct PropertySet {
    values: HashMap<PropertyId, (usize, PropertyValue)>,
}

impl PropertySet {
    pub(crate) fn parse(reader: Reader) -> PropertySet {
        let count = reader.get_u16_le();

        let property_ids: Vec<_> = (0..count).map(|_| PropertyId::parse(reader)).collect();

        let values = property_ids
            .into_iter()
            .enumerate()
            .map(|(idx, id)| (id, (idx, PropertyValue::parse(id, reader))))
            .collect();

        PropertySet { values }
    }

    pub(crate) fn get(&self, id: PropertyId) -> Option<&PropertyValue> {
        self.values.get(&id).map(|(_, value)| value)
    }

    pub(crate) fn index(&self, id: PropertyId) -> Option<usize> {
        self.values.get(&id).map(|(index, _)| index).copied()
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &PropertyValue> {
        self.values.values().map(|(_, value)| value)
    }

    pub(crate) fn values_with_index(&self) -> impl Iterator<Item = &(usize, PropertyValue)> {
        self.values.values()
    }
}
