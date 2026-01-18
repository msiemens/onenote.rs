use crate::shared::{cell_id::CellId, compact_id::CompactId, exguid::ExGuid};
use std::fmt;

pub trait MappingTable {
    fn resolve_id(&self, index: usize, cid: &CompactId) -> Option<ExGuid>;
    fn get_object_space(&self, index: usize, cid: &CompactId) -> Option<CellId>;
}

impl fmt::Debug for dyn MappingTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[MappingTable]")
    }
}
