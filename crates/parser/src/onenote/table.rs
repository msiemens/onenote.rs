use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::color::Color;
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property::outline_indent_distance::OutlineIndentDistance;
use crate::one::property_set::{table_cell_node, table_node, table_row_node};
use crate::onenote::ParserContext;
use crate::onenote::note_tag::{NoteTag, parse_note_tags};
use crate::onenote::outline::{OutlineElement, parse_table_cell_child};
use crate::onestore::ObjectSpace;

/// A table.
///
/// See [\[MS-ONE\] 2.2.26].
///
/// [\[MS-ONE\] 2.2.26]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/9046980a-2410-4b2d-8a35-ec06e55648e0
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
    /// The number of rows in this table.
    pub fn rows(&self) -> u32 {
        self.rows
    }

    /// The number of columns in this table.
    pub fn cols(&self) -> u32 {
        self.cols
    }

    /// The table rows.
    pub fn contents(&self) -> &[TableRow] {
        &self.contents
    }

    /// Which columns have a locked width.
    ///
    /// To determine if column `c` has a locked width, calculate:
    ///
    /// ```ignore
    /// table.cols_locked()[c / 8] & (1 << (c % 8)) == 1;
    /// ```
    ///
    /// See [\[MS-ONE\] 2.3.70].
    ///
    /// [\[MS-ONE\] 2.3.70]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/465c1f7c-63be-4d8b-a8de-76924afe92c2
    pub fn cols_locked(&self) -> &[u8] {
        &self.cols_locked
    }

    /// The column widths in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.66].
    ///
    /// [\[MS-ONE\] 2.3.66]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/aea7e232-e7f3-444b-8d2c-e8f46fa8cc59
    pub fn col_widths(&self) -> &[f32] {
        &self.col_widths
    }

    /// Whether the table borders are visible.
    ///
    /// See [\[MS-ONE\] 2.3.65].
    ///
    /// [\[MS-ONE\] 2.3.65]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/37be8f3c-e015-4c45-be99-615a669439b8
    pub fn borders_visible(&self) -> bool {
        self.borders_visible
    }

    /// The table's alignment relative to the containing outline element (if present).
    ///
    /// See [\[MS-ONE\] 2.3.27].
    ///
    /// [\[MS-ONE\] 2.3.27]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/61fa50be-c355-4b8d-ac01-761a2f7f66c0
    pub fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    /// The table's alignment.
    ///
    /// See [\[MS-ONE\] 2.3.33].
    ///
    /// [\[MS-ONE\] 2.3.33]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/4e7fe9db-2fdb-4239-b291-dc4b909c94ad
    pub fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    /// Note tags for this table.
    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }
}

/// A table row.
///
/// See [\[MS-ONE\] 2.2.27].
///
/// [\[MS-ONE\] 2.2.27]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/d22af1aa-5e0b-40ed-b914-f6397979d6b0
#[derive(Clone, Debug)]
pub struct TableRow {
    pub(crate) contents: Vec<TableCell>,
}

impl TableRow {
    /// The cells in the table row.
    pub fn contents(&self) -> &[TableCell] {
        &self.contents
    }
}

/// A table cell.
///
/// See [\[MS-ONE\] 2.2.28].
///
/// [\[MS-ONE\] 2.2.28]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/e5660d6b-72c3-4d9f-bad0-435c00f42183
#[derive(Clone, Debug)]
pub struct TableCell {
    pub(crate) contents: Vec<OutlineElement>,

    pub(crate) background_color: Option<Color>,
    pub(crate) layout_max_width: Option<f32>,
    pub(crate) outline_indent_distance: OutlineIndentDistance,
}

impl TableCell {
    /// The contents of the table cell.
    pub fn contents(&self) -> &[OutlineElement] {
        &self.contents
    }

    /// The content's max width in half-inch increments.
    ///
    /// See [\[MS-ONE\] 2.3.22].
    ///
    /// [\[MS-ONE\] 2.3.22]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/a770ac4b-2225-4aa6-ba92-d3a51f97c405
    pub fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    /// The indentation size for the table cell contents.
    ///
    /// See [\[MS-ONE\] 2.2.2].
    ///
    /// [\[MS-ONE\] 2.2.2]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/269a3e7b-d85a-4ba8-8e1d-d85e1c840772
    pub fn outline_indent_distance(&self) -> &OutlineIndentDistance {
        &self.outline_indent_distance
    }

    /// The cell's background color.
    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }
}

pub(crate) fn parse_table(
    table_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<Table> {
    let table_object = space
        .get_object(table_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("table object is missing".into()))?;
    let data = table_node::parse(table_object)?;

    let contents = data
        .rows
        .into_iter()
        .map(|row_id| parse_row(row_id, space, ctx))
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
        note_tags: parse_note_tags(&data.note_tags, space)?,
    };

    Ok(table)
}

fn parse_row(
    row_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<TableRow> {
    let row_object = space
        .get_object(row_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("row object is missing".into()))?;
    let data = table_row_node::parse(row_object)?;

    let contents = data
        .cells
        .into_iter()
        .map(|cell_id| parse_cell(cell_id, space, ctx))
        .collect::<Result<_>>()?;

    let row = TableRow { contents };

    Ok(row)
}

fn parse_cell(
    cell_id: ExGuid,
    space: &(impl ObjectSpace + ?Sized),
    ctx: &mut ParserContext,
) -> Result<TableCell> {
    let cell_object = space
        .get_object(cell_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("cell object is missing".into()))?;
    let data = table_cell_node::parse(cell_object)?;

    let mut contents = Vec::new();
    for child_id in data.contents {
        parse_table_cell_child(child_id, space, ctx, &mut contents)?;
    }

    let cell = TableCell {
        contents,
        background_color: data.background_color,
        layout_max_width: data.layout_max_width,
        outline_indent_distance: data.outline_indent_distance,
    };

    Ok(cell)
}

#[cfg(test)]
mod tests {
    use crate::fs::FileSource;
    use crate::fs::file_source::BytesSource;
    use crate::fsshttpb::data::cell_id::CellId;
    use crate::fsshttpb::data::exguid::ExGuid;
    use crate::one::property::PropertyType;
    use crate::one::property_set::{
        PropertySetId, page_series_node, section_node, table_cell_node,
    };
    use crate::onenote::ParserContext;
    use crate::onenote::content::Content;
    use crate::onenote::outline::{OutlineElement, OutlineItem};
    use crate::onenote::parse_store_auto;
    use crate::onenote::section::{Section, parse_section};
    use crate::onestore::shared::compact_id::CompactId;
    use crate::onestore::shared::object_prop_set::ObjectPropSet;
    use crate::onestore::shared::prop_set::PropertySet;
    use crate::onestore::{MappingTable, Object, ObjectSpace, OneStore, OneStoreType};
    use crate::reader::Reader;
    use crate::shared::guid::Guid;
    use crate::warn::Report;
    use bytes::Bytes;
    use std::cell::Cell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::Arc;

    /// The id the synthetic group's one child resolves to; the page space answers it with the
    /// cell's original first element.
    fn alias() -> ExGuid {
        ExGuid::from_guid(
            Guid::from_str("{6A1F0C4E-2B7D-4E59-9C3A-8D5E1F2A7B60}").unwrap(),
            1,
        )
    }

    struct ToAlias;

    impl MappingTable for ToAlias {
        fn resolve_id(&self, _index: usize, _cid: &CompactId) -> Option<ExGuid> {
            Some(alias())
        }

        fn get_object_space(&self, _index: usize, _cid: &CompactId) -> Option<CellId> {
            None
        }
    }

    /// An outline group (child level 1) whose only child is `alias()`.
    fn outline_group() -> Object {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&(PropertyType::ElementChildNodes as u32).to_le_bytes());
        bytes.extend_from_slice(&(PropertyType::OutlineElementChildLevel as u32).to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.push(1);

        Object {
            context_id: Default::default(),
            jc_id: PropertySetId::OutlineGroup.as_jcid(),
            props: ObjectPropSet {
                object_ids: vec![CompactId::parse(&mut Reader::new(&[0; 4])).unwrap()],
                properties: PropertySet::parse(&mut Reader::new(&bytes)).unwrap(),
                ..Default::default()
            },
            file_data: None,
            mapping: Rc::new(ToAlias),
        }
    }

    /// A page space in which the first table cell seen has its first element wrapped in an
    /// outline group, as OneNote for Mac writes it.
    #[derive(Debug)]
    struct GroupedCell<'a> {
        inner: &'a dyn ObjectSpace,
        target: Cell<Option<ExGuid>>,
        group: Object,
    }

    impl ObjectSpace for GroupedCell<'_> {
        fn get_object(&self, id: ExGuid) -> Option<&Object> {
            if id == alias() {
                return self.inner.get_object(self.target.get()?);
            }
            if Some(id) == self.target.get() {
                return Some(&self.group);
            }

            let object = self.inner.get_object(id)?;
            if self.target.get().is_none() && object.id() == PropertySetId::TableCellNode.as_jcid()
            {
                let first = table_cell_node::parse(object)
                    .unwrap()
                    .contents
                    .first()
                    .copied();
                self.target.set(first);
            }

            Some(object)
        }

        fn content_root(&self) -> Option<ExGuid> {
            self.inner.content_root()
        }

        fn metadata_root(&self) -> Option<ExGuid> {
            self.inner.metadata_root()
        }
    }

    #[derive(Debug)]
    struct GroupedCells<'a> {
        inner: &'a dyn OneStore,
        spaces: HashMap<CellId, GroupedCell<'a>>,
    }

    impl<'a> GroupedCells<'a> {
        fn new(inner: &'a dyn OneStore) -> Self {
            let mut ctx = ParserContext {
                page: None,
                report: Report::new(),
                recognized_words: Default::default(),
            };
            let root = inner.data_root();
            let content = root.get_object(root.content_root().unwrap()).unwrap();
            let spaces = section_node::parse(content, &mut ctx)
                .unwrap()
                .page_series
                .into_iter()
                .flat_map(|id| {
                    page_series_node::parse(root.get_object(id).unwrap())
                        .unwrap()
                        .page_spaces
                })
                .filter_map(|id| {
                    let space = inner.object_space(id)?;
                    let grouped = GroupedCell {
                        inner: space,
                        target: Cell::new(None),
                        group: outline_group(),
                    };
                    Some((id, grouped))
                })
                .collect();

            GroupedCells { inner, spaces }
        }

        fn grouped(&self) -> usize {
            self.spaces
                .values()
                .filter(|space| space.target.get().is_some())
                .count()
        }
    }

    impl OneStore for GroupedCells<'_> {
        fn file_identity(&self) -> Guid {
            self.inner.file_identity()
        }

        fn get_type(&self) -> crate::errors::Result<OneStoreType> {
            self.inner.get_type()
        }

        fn data_root(&self) -> &dyn ObjectSpace {
            self.inner.data_root()
        }

        fn object_space(&self, id: CellId) -> Option<&dyn ObjectSpace> {
            match self.spaces.get(&id) {
                Some(space) => Some(space),
                None => self.inner.object_space(id),
            }
        }
    }

    /// The number of elements in every table cell of the section, in document order.
    fn cell_sizes(section: &Section) -> Vec<usize> {
        fn walk_element(element: &OutlineElement, out: &mut Vec<usize>) {
            for content in element.contents() {
                if let Content::Table(table) = content {
                    for row in table.contents() {
                        for cell in row.contents() {
                            out.push(cell.contents().len());
                            cell.contents().iter().for_each(|e| walk_element(e, out));
                        }
                    }
                }
            }
        }

        fn walk_items(items: &[OutlineItem], out: &mut Vec<usize>) {
            for item in items {
                match item {
                    OutlineItem::Element(e) => walk_element(e, out),
                    OutlineItem::Group(group) => walk_items(group.outlines(), out),
                }
            }
        }

        let mut out = Vec::new();
        for series in section.page_series() {
            for page in series.pages() {
                for content in page.contents() {
                    if let Some(outline) = content.outline() {
                        walk_items(outline.items(), &mut out);
                    }
                }
            }
        }

        out
    }

    #[test]
    fn flattens_an_outline_group_inside_a_table_cell() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/samples/joplin/checkboxes_and_unicode.one"
        );
        let data = std::fs::read(path).unwrap();
        let source: Arc<dyn FileSource> = Arc::new(BytesSource::new(Bytes::copy_from_slice(&data)));
        let parsed = parse_store_auto(source).unwrap();
        let inner = parsed.as_onestore();

        let plain = parse_section(inner, "checkboxes_and_unicode.one".into()).unwrap();
        let sizes = cell_sizes(&plain);
        assert!(!sizes.is_empty(), "the sample has a table");

        let store = GroupedCells::new(inner);
        let grouped = parse_section(&store, "checkboxes_and_unicode.one".into())
            .expect("a group inside a table cell must not fail the page");

        assert!(store.grouped() > 0, "no table cell was given a group");
        let flattened = grouped
            .report()
            .warnings()
            .iter()
            .filter(|w| w.message().contains("table cell contains an outline group"))
            .count();
        assert_eq!(flattened, store.grouped());
        // The group's element lands back in its cell, so every cell keeps its contents.
        assert_eq!(cell_sizes(&grouped), sizes);
    }
}
