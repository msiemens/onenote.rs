use crate::errors::{ErrorKind, Result};
use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{image_node, picture_container};
use crate::onenote::parser::note_tag::{parse_note_tags, NoteTag};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Image {
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) extension: Option<String>,

    pub(crate) layout_max_width: Option<f32>,
    pub(crate) layout_max_height: Option<f32>,

    // pub(crate) language_code: Option<u32>,
    pub(crate) alt_text: Option<String>,

    pub(crate) layout_alignment_in_parent: Option<LayoutAlignment>,
    pub(crate) layout_alignment_self: Option<LayoutAlignment>,

    pub(crate) image_filename: Option<String>,

    pub(crate) displayed_page_number: Option<u32>,

    pub(crate) text: Option<String>,
    pub(crate) text_language_code: Option<u32>,

    pub(crate) picture_width: Option<f32>,
    pub(crate) picture_height: Option<f32>,

    pub(crate) hyperlink_url: Option<String>,

    pub(crate) offset_from_parent_horizontal: Option<f32>,
    pub(crate) offset_from_parent_vertical: Option<f32>,

    pub(crate) is_background: bool,

    pub(crate) note_tags: Vec<NoteTag>,
}

impl Image {
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    pub fn layout_max_width(&self) -> Option<f32> {
        self.layout_max_width
    }

    pub fn layout_max_height(&self) -> Option<f32> {
        self.layout_max_height
    }

    pub fn alt_text(&self) -> Option<&str> {
        self.alt_text.as_deref()
    }

    pub fn layout_alignment_in_parent(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_in_parent
    }

    pub fn layout_alignment_self(&self) -> Option<LayoutAlignment> {
        self.layout_alignment_self
    }

    pub fn image_filename(&self) -> Option<&str> {
        self.image_filename.as_deref()
    }

    pub fn displayed_page_number(&self) -> Option<u32> {
        self.displayed_page_number
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub fn text_language_code(&self) -> Option<u32> {
        self.text_language_code
    }

    pub fn picture_width(&self) -> Option<f32> {
        self.picture_width
    }

    pub fn picture_height(&self) -> Option<f32> {
        self.picture_height
    }

    pub fn hyperlink_url(&self) -> Option<&str> {
        self.hyperlink_url.as_deref()
    }

    pub fn offset_from_parent_horizontal(&self) -> Option<f32> {
        self.offset_from_parent_horizontal
    }

    pub fn offset_from_parent_vertical(&self) -> Option<f32> {
        self.offset_from_parent_vertical
    }

    pub fn is_background(&self) -> bool {
        self.is_background
    }

    pub fn note_tags(&self) -> &[NoteTag] {
        &self.note_tags
    }
}

pub(crate) fn parse_image(image_id: ExGuid, space: &ObjectSpace) -> Result<Image> {
    let node_object = space
        .get_object(image_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("image is missing".into()))?;
    let node = image_node::parse(node_object)?;

    let container_data = node
        .picture_container
        .map(|container_object_id| {
            space
                .get_object(container_object_id)
                .ok_or_else(|| ErrorKind::MalformedOneNoteData("image container is missing".into()))
        })
        .transpose()?
        .map(|container_object| picture_container::parse(container_object))
        .transpose()?;

    let (data, extension) = if let Some(data) = container_data {
        (Some(data.data), data.extension)
    } else {
        (None, None)
    };

    // TODO: Parse language code

    let image = Image {
        data,
        extension,
        layout_max_width: node.layout_max_width,
        layout_max_height: node.layout_max_height,
        alt_text: node.alt_text.map(String::from),
        layout_alignment_in_parent: node.layout_alignment_in_parent,
        layout_alignment_self: node.layout_alignment_self,
        image_filename: node.image_filename,
        displayed_page_number: node.displayed_page_number,
        text: node.text.map(String::from),
        text_language_code: node.text_language_code,
        picture_width: node.picture_width,
        picture_height: node.picture_height,
        hyperlink_url: node.hyperlink_url.map(String::from),
        offset_from_parent_horizontal: node.offset_from_parent_horiz,
        offset_from_parent_vertical: node.offset_from_parent_vert,
        is_background: node.is_background,
        note_tags: parse_note_tags(node.note_tags, space)?,
    };

    Ok(image)
}
