use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::object_stream_header::ObjectStreamHeader;
use crate::onestore::types::prop_set::PropertySet;
use crate::onestore::types::property::{PropertyId, PropertyValue};
use crate::Reader;

#[derive(Debug)]
pub(crate) struct ObjectPropSet {
    pub(crate) object_ids: Vec<CompactId>,
    pub(crate) object_space_ids: Vec<CompactId>,
    pub(crate) context_ids: Vec<CompactId>,
    properties: PropertySet,
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

    pub(crate) fn get(&self, id: u32) -> Option<&PropertyValue> {
        self.properties.get(PropertyId::new(id))
    }
}
