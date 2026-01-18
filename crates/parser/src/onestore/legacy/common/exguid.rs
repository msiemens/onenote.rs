use crate::errors::Result;
use crate::onestore::legacy::parse::Parse;
use crate::shared::guid::Guid;
use crate::{Reader, fsshttpb};
use parser_macros::Parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Parse)]
pub(crate) struct ExGuid {
    pub guid: Guid,
    pub value: u32,
}

impl ExGuid {
    pub(crate) fn parse(reader: Reader) -> Result<Self> {
        <Self as Parse>::parse(reader)
    }
}

impl From<fsshttpb::data::exguid::ExGuid> for ExGuid {
    fn from(value: fsshttpb::data::exguid::ExGuid) -> Self {
        Self {
            value: value.value,
            guid: value.guid,
        }
    }
}

impl From<ExGuid> for fsshttpb::data::exguid::ExGuid {
    fn from(value: ExGuid) -> Self {
        fsshttpb::data::exguid::ExGuid {
            guid: value.guid,
            value: value.value,
        }
    }
}
