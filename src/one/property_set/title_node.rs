use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified_time: Time,
    children: Vec<ExGuid>,
    offset_horizontal: f32,
    offset_vertical: f32,
    layout_alignment_in_parent: Option<LayoutAlignment>,
    layout_alignment_self: Option<LayoutAlignment>,
}

impl Data {
    pub(crate) fn children(&self) -> &[ExGuid] {
        &self.children
    }

    pub(crate) fn offset_horizontal(&self) -> f32 {
        self.offset_horizontal
    }

    pub(crate) fn offset_vertical(&self) -> f32 {
        self.offset_vertical
    }

    pub(crate) fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub(crate) fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TitleNode.as_jcid());

    let last_modified_time = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("title node has no last_modified time");

    let children = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)
        .expect("title node has no child nodes");
    let offset_horizontal = simple::parse_f32(PropertyType::OffsetFromParentHoriz, object)
        .expect("title has no horizontal offset");
    let offset_vertical = simple::parse_f32(PropertyType::OffsetFromParentVert, object)
        .expect("title has no vertical offset");

    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object);
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object);

    Data {
        last_modified_time,
        children,
        offset_horizontal,
        offset_vertical,
        layout_alignment_in_parent,
        layout_alignment_self,
    }
}
