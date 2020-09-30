use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::note_tag_container::Data as NoteTagData;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) last_modified: Time,
    pub(crate) rows: Vec<ExGuid>,
    pub(crate) row_count: u32,
    pub(crate) col_count: u32,
    pub(crate) cols_locked: Vec<u8>,
    pub(crate) col_widths: Vec<f32>,
    pub(crate) borders_visible: bool,
    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,
    pub(crate) note_tags: Vec<NoteTagData>,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::TableNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("table has no last modified time");
    let rows = ObjectReference::parse_vec(PropertyType::ElementChildNodes, object)
        .expect("table has no rows");
    let row_count =
        simple::parse_u32(PropertyType::RowCount, object).expect("table has no row count");
    let col_count =
        simple::parse_u32(PropertyType::ColumnCount, object).expect("table has no col count");
    let cols_locked = simple::parse_vec(PropertyType::TableColumnsLocked, object)
        .map(|value| value.into_iter().skip(1).collect())
        .unwrap_or_default();
    let col_widths = simple::parse_vec(PropertyType::TableColumnWidths, object)
        .map(|value| {
            value
                .into_iter()
                .skip(1)
                .collect::<Vec<_>>()
                .chunks_exact(4)
                .map(|v| f32::from_le_bytes([v[0], v[1], v[2], v[3]]))
                .collect()
        })
        .unwrap_or_default();
    let borders_visible =
        simple::parse_bool(PropertyType::TableBordersVisible, object).unwrap_or(true);
    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object);
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object);

    let note_tags = NoteTagData::parse(object).unwrap_or_default();

    Data {
        last_modified,
        rows,
        row_count,
        col_count,
        cols_locked,
        col_widths,
        borders_visible,
        layout_alignment_in_parent,
        layout_alignment_self,
        note_tags,
    }
}
