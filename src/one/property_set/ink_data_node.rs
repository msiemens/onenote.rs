use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::{PropertyType, simple};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::object::Object;

/// An ink data container.
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) strokes: Vec<ExGuid>,
    pub(crate) bounding_box: Option<[u32; 4]>,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::InkDataNode)?;

    let strokes =
        ObjectReference::parse_vec(PropertyType::InkStrokes, object)?.ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("ink data node has no strokes".into())
        })?;
    let bounding_box = simple::parse_vec_u32(PropertyType::InkBoundingBox, object)?
        .filter(|values| values.len() == 4)
        .map(|values| [values[0], values[1], values[2], values[3]]);

    Ok(Data {
        strokes,
        bounding_box,
    })
}
