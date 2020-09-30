use crate::one::property::layout_alignment::LayoutAlignment;
use crate::one::property_set::{image_node, picture_container};
use crate::onenote::parser::note_tag::{parse_note_tags, NoteTag};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
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

pub(crate) fn parse_image(image_id: ExGuid, space: &ObjectSpace) -> Image {
    let node_object = space.get_object(image_id).expect("image is missing");
    let node = image_node::parse(node_object);

    let container_data = node
        .picture_container
        .map(|container_object_id| {
            space
                .get_object(container_object_id)
                .expect("image container is missing")
        })
        .map(|container_object| picture_container::parse(container_object))
        .map(|container| container);

    let (data, extension) = if let Some(data) = container_data {
        (Some(data.data), data.extension)
    } else {
        (None, None)
    };

    // TODO: Parse language code

    Image {
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
        note_tags: parse_note_tags(node.note_tags, space),
    }
}
