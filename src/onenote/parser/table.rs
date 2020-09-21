use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::outline_node::OutlineIndentDistance;
use crate::one::property_set::{table_cell_node, table_node, table_row_node};
use crate::onenote::parser::outline::{parse_outline_element, OutlineElement};
use crate::onestore::object_space::ObjectSpace;
use crate::onestore::revision::Revision;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct Table {
    pub(crate) rows: u32,
    pub(crate) cols: u32,
    pub(crate) contents: Vec<TableRow>,

    pub(crate) cols_locked: Vec<u8>,
    pub(crate) col_widths: Vec<f32>,

    pub(crate) borders_visible: bool,

    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,
}

#[derive(Debug)]
pub struct TableRow {
    pub(crate) contents: Vec<TableCell>,
}

#[derive(Debug)]
pub struct TableCell {
    pub(crate) contents: Vec<OutlineElement>,

    pub(crate) layout_max_width: Option<f32>,
    pub(crate) outline_indent_distance: OutlineIndentDistance,
}

pub(crate) fn parse_table(table_id: ExGuid, rev: &Revision, space: &ObjectSpace) -> Table {
    let table_object = rev
        .resolve_object(table_id, space)
        .expect("table object is missing");
    let data = table_node::parse(table_object);

    let contents = data
        .rows()
        .iter()
        .map(|id| parse_row(*id, rev, space))
        .collect();

    Table {
        rows: data.row_count(),
        cols: data.col_count(),
        contents,
        cols_locked: data.cols_locked().to_vec(),
        col_widths: data.col_widths().to_vec(),
        borders_visible: data.borders_visible(),
        layout_alignment_in_parent: data.layout_alignment_in_parent(),
        layout_alignment_self: data.layout_alignment_self(),
    }
}

fn parse_row(row_id: ExGuid, rev: &Revision, space: &ObjectSpace) -> TableRow {
    let row_object = rev
        .resolve_object(row_id, space)
        .expect("row object is missing");
    let data = table_row_node::parse(row_object);

    let contents = data
        .cells()
        .iter()
        .map(|id| parse_cell(*id, rev, space))
        .collect();

    TableRow { contents }
}

fn parse_cell(cell_id: ExGuid, rev: &Revision, space: &ObjectSpace) -> TableCell {
    let cell_object = rev
        .resolve_object(cell_id, space)
        .expect("cell object is missing");
    let data = table_cell_node::parse(cell_object);

    let contents = data
        .contents()
        .iter()
        .map(|id| parse_outline_element(*id, rev, space))
        .collect();

    TableCell {
        contents,
        layout_max_width: data.layout_max_width(),
        outline_indent_distance: data.outline_indent_distance().clone(),
    }
}
