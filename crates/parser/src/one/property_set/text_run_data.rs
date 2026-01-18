use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::one::property_set::parse_object;
use crate::onestore::Object;

pub(crate) fn parse(object: &Object) -> Result<Option<Vec<Object>>> {
    let (prop_id, prop_sets) = match object.props.get(PropertyType::TextRunData) {
        Some(value) => value.to_property_values().ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData(
                "text run data is not a property values list".into(),
            )
        })?,
        None => return Ok(None),
    };

    let data = prop_sets
        .iter()
        .map(|props| parse_object(object, prop_id, PropertyType::TextRunData, props))
        .collect::<Result<Vec<_>>>()?;

    Ok(Some(data))
}
