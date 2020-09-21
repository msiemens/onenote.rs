use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::outline_node::OutlineIndentDistance;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified: Time,
    contents: Vec<ExGuid>,
    layout_max_width: Option<f32>,
    outline_indent_distance: OutlineIndentDistance,
}

impl Data {
    pub(crate) fn contents(&self) -> &[ExGuid] {
        &self.contents
    }

    pub(crate) fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    pub(crate) fn outline_indent_distance(&self) -> &OutlineIndentDistance {
        &self.outline_indent_distance
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TableCellNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("table cell has no last modified time");
    let contents = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)
        .expect("table cell has no contents");
    let layout_max_width = simple::parse_f32(PropertyType::LayoutMaxWidth, object);
    let outline_indent_distance =
        OutlineIndentDistance::parse(object).expect("table cell has no outline indent distance");

    Data {
        last_modified,
        contents,
        layout_max_width,
        outline_indent_distance,
    }
}
