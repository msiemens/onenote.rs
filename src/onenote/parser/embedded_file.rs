use crate::one::property_set::embedded_file_node::FileType;
use crate::one::property_set::{embedded_file_container, embedded_file_node};
use crate::onenote::parser::note_tag::{parse_note_tags, NoteTag};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub struct EmbeddedFile {
    pub(crate) filename: String,
    pub(crate) file_type: FileType,
    pub(crate) data: Vec<u8>,

    pub(crate) layout_max_width: Option<f32>,
    pub(crate) layout_max_height: Option<f32>,

    pub(crate) offset_from_parent_horizontal: Option<f32>,
    pub(crate) offset_from_parent_vertical: Option<f32>,

    pub(crate) note_tags: Vec<NoteTag>,
}

impl EmbeddedFile {
    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    pub fn layout_max_height(&self) -> Option<f32> {
        self.layout_max_height
    }

    pub fn offset_from_parent_horizontal(&self) -> Option<f32> {
        self.offset_from_parent_horizontal
    }

    pub fn offset_from_parent_vertical(&self) -> Option<f32> {
        self.offset_from_parent_vertical
    }

    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }
}

pub(crate) fn parse_embedded_file(file_id: ExGuid, space: &ObjectSpace) -> EmbeddedFile {
    let node_object = space.get_object(file_id).expect("embedded file is missing");
    let node = embedded_file_node::parse(node_object);

    let container_object_id = node.embedded_file_container;
    let container_object = space
        .get_object(container_object_id)
        .expect("embedded file container is missing");
    let container = embedded_file_container::parse(container_object);

    // TODO: Resolve picture container

    EmbeddedFile {
        filename: node.embedded_file_name,
        file_type: node.file_type,
        data: container.into_value(),
        layout_max_width: node.layout_max_width,
        layout_max_height: node.layout_max_height,
        offset_from_parent_horizontal: node.offset_from_parent_horiz,
        offset_from_parent_vertical: node.offset_from_parent_vert,
        note_tags: parse_note_tags(node.note_tags, space),
    }
}
