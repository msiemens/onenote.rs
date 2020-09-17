use std::collections::HashMap;

use crate::onestore::types::compact_id::CompactId;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct MappingTable(HashMap<CompactId, ExGuid>);

impl MappingTable {
    pub(crate) fn from_entries<I: Iterator<Item = (CompactId, ExGuid)>>(
        entries: I,
    ) -> MappingTable {
        MappingTable(entries.collect())
    }

    pub(crate) fn get_object(&self, id: CompactId) -> Option<ExGuid> {
        self.0.get(&id).copied()
    }
}
