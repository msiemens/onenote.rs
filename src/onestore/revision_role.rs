use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;

#[derive(Debug, PartialEq, Hash, Eq)]
pub(crate) enum RevisionRole {
    DefaultContent,
    Metadata,
    EncryptionKey,
    VersionMetadata,
}

impl RevisionRole {
    pub(crate) fn parse(id: ExGuid) -> RevisionRole {
        let guid = Guid::from_str("4A3717F8-1C14-49E7-9526-81D942DE1741").expect("invalid GUID");

        if id.guid != guid {
            panic!("invalid root declare root id")
        }

        match id {
            ExGuid { value: 1, .. } => RevisionRole::DefaultContent,
            ExGuid { value: 2, .. } => RevisionRole::Metadata,
            ExGuid { value: 3, .. } => RevisionRole::EncryptionKey,
            ExGuid { value: 4, .. } => RevisionRole::VersionMetadata,
            ExGuid { value, .. } => panic!("invalid root declare id exguid value {}", value),
        }
    }
}
