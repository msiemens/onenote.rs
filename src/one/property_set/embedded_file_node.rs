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
    last_modified: Time,
    picture_container: Option<ExGuid>,
    layout_max_width: Option<f32>,
    layout_max_height: Option<f32>,
    is_layout_size_set_by_user: bool,
    text: Option<String>,
    text_language_code: Option<u32>,
    layout_alignment_in_parent: Option<LayoutAlignment>,
    layout_alignment_self: Option<LayoutAlignment>,
    embedded_file_container: ExGuid,
    embedded_file_name: String,
    source_path: Option<String>,
    file_type: FileType,
    picture_width: Option<f32>,
    picture_height: Option<f32>,
    note_tags: Vec<NoteTagData>,
    offset_from_parent_horiz: Option<f32>,
    offset_from_parent_vert: Option<f32>,
    recording_duration: Option<u32>,
}

impl Data {
    pub(crate) fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    pub(crate) fn layout_max_height(&self) -> Option<f32> {
        self.layout_max_height
    }

    pub(crate) fn embedded_file_container(&self) -> ExGuid {
        self.embedded_file_container
    }

    pub(crate) fn embedded_file_name(&self) -> &str {
        &self.embedded_file_name
    }

    pub(crate) fn offset_from_parent_horiz(&self) -> Option<f32> {
        self.offset_from_parent_horiz
    }

    pub(crate) fn offset_from_parent_vert(&self) -> Option<f32> {
        self.offset_from_parent_vert
    }

    pub fn note_tags(&self) -> &[NoteTagData] {
        &self.note_tags
    }
}

#[derive(Debug)]
pub(crate) enum FileType {
    Unknown,
    Audio,
    Video,
}

impl FileType {
    fn parse(object: &Object) -> FileType {
        object
            .props()
            .get(PropertyType::IRecordMedia)
            .map(|value| value.to_u32().expect("file type is not a u32"))
            .map(|value| match value {
                1 => FileType::Audio,
                2 => FileType::Video,
                _ => panic!("invalid file type: {}", value),
            })
            .unwrap_or(FileType::Unknown)
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::EmbeddedFileNode.as_jcid());

    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("embedded file has no last modified time");
    let picture_container = ObjectReference::parse(PropertyType::PictureContainer, object);
    let layout_max_width = simple::parse_f32(PropertyType::LayoutMaxWidth, object);
    let layout_max_height = simple::parse_f32(PropertyType::LayoutMaxHeight, object);
    let is_layout_size_set_by_user =
        simple::parse_bool(PropertyType::IsLayoutSizeSetByUser, object).unwrap_or_default();
    let text = simple::parse_string(PropertyType::RichEditTextUnicode, object);
    let text_language_code =
        simple::parse_u16(PropertyType::RichEditTextLangID, object).map(|value| value as u32);
    let layout_alignment_in_parent =
        LayoutAlignment::parse(PropertyType::LayoutAlignmentInParent, object);
    let layout_alignment_self = LayoutAlignment::parse(PropertyType::LayoutAlignmentSelf, object);
    let embedded_file_container =
        ObjectReference::parse(PropertyType::EmbeddedFileContainer, object)
            .expect("embedded file has no file container");
    let embedded_file_name = simple::parse_string(PropertyType::EmbeddedFileName, object)
        .expect("embedded file has no file name");
    let source_path = simple::parse_string(PropertyType::SourceFilepath, object);
    let file_type = FileType::parse(object);
    let picture_width = simple::parse_f32(PropertyType::PictureWidth, object);
    let picture_height = simple::parse_f32(PropertyType::PictureHeight, object);
    let offset_from_parent_horiz = simple::parse_f32(PropertyType::OffsetFromParentHoriz, object);
    let offset_from_parent_vert = simple::parse_f32(PropertyType::OffsetFromParentVert, object);
    // let recording_duration = simple::parse_u32(PropertyType::Duration) // FIXME: Record duration property id not known

    let note_tags = NoteTagData::parse(object).unwrap_or_default();

    Data {
        last_modified,
        picture_container,
        layout_max_width,
        layout_max_height,
        is_layout_size_set_by_user,
        text,
        text_language_code,
        layout_alignment_in_parent,
        layout_alignment_self,
        embedded_file_container,
        embedded_file_name,
        source_path,
        file_type,
        picture_width,
        picture_height,
        note_tags,
        offset_from_parent_horiz,
        offset_from_parent_vert,
        recording_duration: None, // FIXME: Parse this
    }
}
