use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::note_tag_state::NoteTagState;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    last_modified: Time,
    rows: Vec<ExGuid>,
    row_count: u32,
    col_count: u32,
    cols_locked: Vec<u8>,
    col_widths: Vec<f32>,
    borders_visible: bool,
    layout_alignment_in_parent: Option<LayoutAlignment>,
    layout_alignment_self: Option<LayoutAlignment>,
    note_tag_states: Vec<NoteTagState>,
}

impl Data {
    pub(crate) fn rows(&self) -> &[ExGuid] {
        &self.rows
    }

    pub(crate) fn row_count(&self) -> u32 {
        self.row_count
    }

    pub(crate) fn col_count(&self) -> u32 {
        self.col_count
    }

    pub(crate) fn cols_locked(&self) -> &[u8] {
        &self.cols_locked
    }

    pub(crate) fn col_widths(&self) -> &[f32] {
        &self.col_widths
    }

    pub(crate) fn borders_visible(&self) -> bool {
        self.borders_visible
    }

    pub(crate) fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub(crate) fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }
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
        .expect("table has no col width definition");
    let borders_visible =
        simple::parse_bool(PropertyType::TableBordersVisible, object).unwrap_or(true);
    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object);
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object);

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
        note_tag_states: vec![], // FIXME: Parse this
    }
}
