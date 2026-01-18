use crate::shared::guid::Guid;
use parser_macros::Parse;

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct GlobalIdTableStartFNDX {
    reserved: u8,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct GlobalIdTableEntryFNDX {
    pub(crate) index: u32,
    pub(crate) guid: Guid,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct GlobalIdTableEntry2FNDX {
    pub(crate) i_index_map_from: u32,
    pub(crate) i_index_map_to: u32,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct GlobalIdTableEntry3FNDX {
    i_index_copy_from_start: u32,
    c_entries_to_copy: u32,
    i_index_copy_to_start: u32,
}
