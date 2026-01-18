use crate::Reader;
use crate::onestore::legacy::ExGuid;
use crate::onestore::legacy::file_node::shared::PointerToListFND;
use crate::onestore::legacy::parse::Parse;
use parser_macros::Parse;

/// See [MS-ONESTORE 2.1.10](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/14af4d81-c2d6-43e6-8bd4-508d4123fb22)
pub(crate) type RevisionManifestListReferenceFND = PointerToListFND;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RevisionManifestListStartFND {
    pub(crate) gsoid: ExGuid,
    n_instance: u32,
}

impl Parse for RevisionManifestListStartFND {
    fn parse(reader: Reader) -> crate::errors::Result<Self> {
        Ok(Self {
            gsoid: ExGuid::parse(reader)?,
            n_instance: reader.get_u32()?,
        })
    }
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RevisionManifestStart4FND {
    pub(crate) rid: ExGuid,
    pub(crate) rid_dependent: ExGuid,
    reserved_time_creation: u64,
    revision_role: u32,
    odcs_default: u16,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RevisionManifestStart6FND {
    pub(crate) rid: ExGuid,
    /// ID of a dependency revision
    pub(crate) rid_dependent: ExGuid,
    revision_role: u32,
    odcs_default: u16,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RevisionManifestStart7FND {
    pub(crate) base: RevisionManifestStart6FND,
    gctxid: ExGuid,
}
