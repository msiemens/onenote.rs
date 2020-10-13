use crate::errors::{ErrorKind, Result};
use crate::types::exguid::ExGuid;

#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
pub(crate) enum RevisionRole {
    DefaultContent,
    Metadata,
    EncryptionKey,
    VersionMetadata,
}

impl RevisionRole {
    pub(crate) fn parse(id: ExGuid) -> Result<RevisionRole> {
        let guid = guid!({4A3717F8-1C14-49E7-9526-81D942DE1741});

        if id.guid != guid {
            return Err(
                ErrorKind::MalformedOneStoreData("invalid root declare root id".into()).into(),
            );
        }

        match id {
            ExGuid { value: 1, .. } => Ok(RevisionRole::DefaultContent),
            ExGuid { value: 2, .. } => Ok(RevisionRole::Metadata),
            ExGuid { value: 3, .. } => Ok(RevisionRole::EncryptionKey),
            ExGuid { value: 4, .. } => Ok(RevisionRole::VersionMetadata),
            ExGuid { value, .. } => Err(ErrorKind::MalformedOneStoreData(
                format!("invalid root declare id exguid value {}", value).into(),
            )
            .into()),
        }
    }
}
