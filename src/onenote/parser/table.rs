use crate::errors::{ErrorKind, Result};
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::outline_node::OutlineIndentDistance;
use crate::one::property_set::{table_cell_node, table_node, table_row_node};
use crate::onenote::parser::note_tag::{parse_note_tags, NoteTag};
use crate::onenote::parser::outline::{parse_outline_element, OutlineElement};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;
use crate::Color;

#[derive(Clone, Debug)]
pub struct Table {
    pub(crate) rows: u32,
    pub(crate) cols: u32,
    pub(crate) contents: Vec<TableRow>,

    pub(crate) cols_locked: Vec<u8>,
    pub(crate) col_widths: Vec<f32>,

    pub(crate) borders_visible: bool,

    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,

    pub(crate) note_tags: Vec<NoteTag>,
}

impl Table {
    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn contents(&self) -> &[TableRow] {
        &self.contents
    }

    pub fn cols_locked(&self) -> &[u8] {
        &self.cols_locked
    }

    pub fn col_widths(&self) -> &[f32] {
        &self.col_widths
    }

    pub fn borders_visible(&self) -> bool {
        self.borders_visible
    }

    pub fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }
}

#[derive(Clone, Debug)]
pub struct TableRow {
    pub(crate) contents: Vec<TableCell>,
}

impl TableRow {
    pub fn contents(&self) -> &[TableCell] {
        &self.contents
    }
}

#[derive(Clone, Debug)]
pub struct TableCell {
    pub(crate) contents: Vec<OutlineElement>,

    pub(crate) background_color: Option<Color>,
    pub(crate) layout_max_width: Option<f32>,
    pub(crate) outline_indent_distance: OutlineIndentDistance,
}

impl TableCell {
    pub fn contents(&self) -> &[OutlineElement] {
        &self.contents
    }

    pub fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    pub fn outline_indent_distance(&self) -> &OutlineIndentDistance {
        &self.outline_indent_distance
    }

    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }
}

pub(crate) fn parse_table(table_id: ExGuid, space: &ObjectSpace) -> Result<Table> {
    let table_object = space
        .get_object(table_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("table object is missing".into()))?;
    let data = table_node::parse(table_object)?;

    let contents = data
        .rows
        .into_iter()
        .map(|row_id| parse_row(row_id, space))
        .collect::<Result<_>>()?;

    let table = Table {
        rows: data.row_count,
        cols: data.col_count,
        contents,
        cols_locked: data.cols_locked,
        col_widths: data.col_widths,
        borders_visible: data.borders_visible,
        layout_alignment_in_parent: data.layout_alignment_in_parent,
        layout_alignment_self: data.layout_alignment_self,
        note_tags: parse_note_tags(data.note_tags, space)?,
    };

    Ok(table)
}

fn parse_row(row_id: ExGuid, space: &ObjectSpace) -> Result<TableRow> {
    let row_object = space
        .get_object(row_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("row object is missing".into()))?;
    let data = table_row_node::parse(row_object)?;

    let contents = data
        .cells
        .into_iter()
        .map(|cell_id| parse_cell(cell_id, space))
        .collect::<Result<_>>()?;

    let row = TableRow { contents };

    Ok(row)
}

fn parse_cell(cell_id: ExGuid, space: &ObjectSpace) -> Result<TableCell> {
    let cell_object = space
        .get_object(cell_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("cell object is missing".into()))?;
    let data = table_cell_node::parse(cell_object)?;

    let contents = data
        .contents
        .into_iter()
        .map(|element_id| parse_outline_element(element_id, space))
        .collect::<Result<_>>()?;

    let cell = TableCell {
        contents,
        background_color: data.background_color,
        layout_max_width: data.layout_max_width,
        outline_indent_distance: data.outline_indent_distance,
    };

    Ok(cell)
}
