use crate::one::property::PropertyType;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::object_stream_header::ObjectStreamHeader;
use crate::onestore::types::prop_set::PropertySet;
use crate::onestore::types::property::{PropertyId, PropertyValue};
use crate::Reader;

#[derive(Debug)]
pub(crate) struct ObjectPropSet {
    object_ids: Vec<CompactId>,
    object_space_ids: Vec<CompactId>,
    context_ids: Vec<CompactId>,
    properties: PropertySet,
}

impl ObjectPropSet {
    pub(crate) fn object_ids(&self) -> &[CompactId] {
        &self.object_ids
    }

    pub(crate) fn object_space_ids(&self) -> &[CompactId] {
        &self.object_space_ids
    }

    pub(crate) fn context_ids(&self) -> &[CompactId] {
        &self.context_ids
    }

    pub(crate) fn properties(&self) -> &PropertySet {
        &self.properties
    }
}

impl ObjectPropSet {
    pub(crate) fn parse(reader: Reader) -> ObjectPropSet {
        let header = ObjectStreamHeader::parse(reader);
        let object_ids = (0..header.count)
            .map(|_| CompactId::parse(reader))
            .collect();

        let mut object_space_ids = vec![];
        let mut context_ids = vec![];

        if !header.osid_stream_not_present {
            let header = ObjectStreamHeader::parse(reader);

            object_space_ids = (0..header.count)
                .map(|_| CompactId::parse(reader))
                .collect();

            if header.extended_streams_present {
                let header = ObjectStreamHeader::parse(reader);
                context_ids = (0..header.count)
                    .map(|_| CompactId::parse(reader))
                    .collect();
            };
        }

        let properties = PropertySet::parse(reader);

        ObjectPropSet {
            object_ids,
            object_space_ids,
            context_ids,
            properties,
        }
    }

    pub(crate) fn get(&self, prop_type: PropertyType) -> Option<&PropertyValue> {
        self.properties.get(PropertyId::new(prop_type as u32))
    }
}
