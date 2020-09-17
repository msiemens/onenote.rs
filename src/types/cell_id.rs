use crate::types::compact_u64::CompactU64;
use crate::types::exguid::ExGuid;
use crate::Reader;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellId(pub ExGuid, pub ExGuid);

impl CellId {
    pub(crate) fn parse(reader: Reader) -> CellId {
        let first = ExGuid::parse(reader);
        let second = ExGuid::parse(reader);

        CellId(first, second)
    }

    pub(crate) fn parse_array(reader: Reader) -> Vec<CellId> {
        let mut values = vec![];

        let count = CompactU64::parse(reader).value();
        for _ in 0..count {
            values.push(CellId::parse(reader));
        }

        values
    }
}
