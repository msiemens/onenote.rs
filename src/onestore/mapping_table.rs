use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::types::compact_id::CompactId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct MappingTable {
    objects: HashMap<CompactId, ExGuid>,
    object_spaces: HashMap<CompactId, CellId>,
}

impl MappingTable {
    pub(crate) fn from_entries<
        I: Iterator<Item = (CompactId, ExGuid)>,
        J: Iterator<Item = (CompactId, CellId)>,
    >(
        objects: I,
        object_spaces: J,
    ) -> MappingTable {
        MappingTable {
            objects: objects.collect(),
            object_spaces: object_spaces.collect(),
        }
    }

    pub(crate) fn get_object(&self, id: CompactId) -> Option<ExGuid> {
        self.objects.get(&id).copied()
    }

    pub(crate) fn get_object_space(&self, id: CompactId) -> Option<CellId> {
        self.object_spaces.get(&id).copied()
    }
}
