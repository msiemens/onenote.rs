use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::reader::Reader;
use crate::shared::guid::Guid;

/// The dimensions (X or Y) for an ink stoke with lower and upper limits.
#[allow(dead_code)]
pub(crate) struct InkDimension {
    pub(crate) id: Guid,
    pub(crate) limit_lower: i32,
    pub(crate) limit_upper: i32,
}

impl InkDimension {
    pub(crate) fn parse(prop_type: PropertyType, object: &Object) -> Result<Vec<InkDimension>> {
        object
            .props()
            .get(prop_type)
            .map(|value| {
                value.to_vec().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("ink dimensions is not a vec".into())
                })
            })
            .transpose()?
            .iter()
            .flat_map(|data| data.chunks_exact(32))
            .map(|entry| {
                let mut reader = Reader::new(entry);
                let id = Guid::parse(&mut reader)?;
                let limit_lower = reader.get_u32()? as i32;
                let limit_upper = reader.get_u32()? as i32;

                Ok(InkDimension {
                    id,
                    limit_lower,
                    limit_upper,
                })
            })
            .collect::<Result<Vec<_>>>()
    }
}
