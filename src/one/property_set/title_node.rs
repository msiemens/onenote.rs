use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{PropertyType, simple};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::object::Object;

/// A page title.
///
/// See [\[MS-ONE\] 2.2.29].
///
/// [\[MS-ONE\] 2.2.29]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/08bd4fd5-59fb-4568-9c82-d2d5280eced8
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) last_modified_time: Time,
    pub(crate) children: Vec<ExGuid>,
    pub(crate) offset_horizontal: f32,
    pub(crate) offset_vertical: f32,
    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::TitleNode)?;

    let last_modified_time =
        Time::parse(PropertyType::LastModifiedTime, object)?.ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("title node has no last_modified time".into())
        })?;

    let children = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("title node has no child nodes".into())
        })?;
    let offset_horizontal = simple::parse_f32(PropertyType::OffsetFromParentHoriz, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("title has no horizontal offset".into())
        })?;
    let offset_vertical = simple::parse_f32(PropertyType::OffsetFromParentVert, object)?
        .ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("title has no vertical offset".into())
        })?;

    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object)?;
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object)?;

    let data = Data {
        last_modified_time,
        children,
        offset_horizontal,
        offset_vertical,
        layout_alignment_in_parent,
        layout_alignment_self,
    };

    Ok(data)
}
