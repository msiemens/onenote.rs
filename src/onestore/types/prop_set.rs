use crate::errors::Result;
use crate::onestore::types::property::{PropertyId, PropertyValue};
use crate::Reader;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct PropertySet {
    values: HashMap<u32, (usize, PropertyValue)>,
}

impl PropertySet {
    pub(crate) fn parse(reader: Reader) -> Result<PropertySet> {
        let count = reader.get_u16()?;

        let property_ids: Vec<_> = (0..count)
            .map(|_| PropertyId::parse(reader))
            .collect::<Result<_>>()?;

        let values = property_ids
            .into_iter()
            .enumerate()
            .map(|(idx, id)| Ok((id.id(), (idx, PropertyValue::parse(id, reader)?))))
            .collect::<Result<_>>()?;

        Ok(PropertySet { values })
    }

    pub(crate) fn get(&self, id: PropertyId) -> Option<&PropertyValue> {
        self.values.get(&id.id()).map(|(_, value)| value)
    }

    pub(crate) fn index(&self, id: PropertyId) -> Option<usize> {
        self.values.get(&id.id()).map(|(index, _)| index).copied()
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &PropertyValue> {
        self.values.values().map(|(_, value)| value)
    }

    pub(crate) fn values_with_index(&self) -> impl Iterator<Item = &(usize, PropertyValue)> {
        self.values.values()
    }
}
