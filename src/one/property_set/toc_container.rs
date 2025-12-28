use crate::errors::Result;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::{PropertyType, simple};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::object::Object;
use crate::property::common::Color;

/// A section's table of contents.
///
/// See [\[MS-ONE\] 2.2.15].
///
/// [\[MS-ONE\] 2.2.15]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/6c1dd264-850b-4e46-af62-50b4dba49b62
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) children: Vec<ExGuid>,
    pub(crate) filename: Option<String>,
    pub(crate) ordering_id: Option<u32>,
    pub(crate) color: Option<Color>,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::TocContainer)?;

    let children =
        ObjectReference::parse_vec(PropertyType::TocChildren, object)?.unwrap_or_default();
    let filename = simple::parse_string(PropertyType::FolderChildFilename, object)?
        .map(|s| s.replace("^M", "+"))
        .map(|s| s.replace("^J", ","));
    let ordering_id = simple::parse_u32(PropertyType::NotebookElementOrderingId, object)?;
    let color = Color::parse(PropertyType::SectionColor, object)?;

    Ok(Data {
        children,
        filename,
        ordering_id,
        color,
    })
}
