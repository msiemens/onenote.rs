use crate::onestore::legacy::ExGuid;
use crate::onestore::shared::compact_id::CompactId;
use parser_macros::Parse;

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RootObjectReference2FNDX {
    pub(crate) oid_root: CompactId,
    pub(crate) root_role: u32,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RootObjectReference3FND {
    pub(crate) oid_root: ExGuid,
    pub(crate) root_role: u32,
}
