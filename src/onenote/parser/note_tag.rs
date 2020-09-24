use crate::one::property::color_ref::ColorRef;
use crate::one::property::note_tag::{ActionItemStatus, ActionItemType};
use crate::one::property::time::Time;
use crate::one::property_set::note_tag_container::Data;
use crate::one::property_set::note_tag_shared_definition_container;
use crate::one::property_set::note_tag_shared_definition_container::{
    NoteTagPropertyStatus, NoteTagShape,
};
use crate::onestore::object_space::ObjectSpace;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct NoteTag {
    completed_at: Option<Time>,
    item_status: ActionItemStatus,
    definition: Option<NoteTagDefinition>,
}

#[derive(Debug)]
pub(crate) struct NoteTagDefinition {
    label: String,
    status: NoteTagPropertyStatus,
    shape: NoteTagShape,
    highlight_color: Option<ColorRef>,
    text_color: Option<ColorRef>,
    action_item_type: ActionItemType,
}

pub(crate) fn parse_note_tags(note_tags: &[Data], space: &ObjectSpace) -> Vec<NoteTag> {
    note_tags
        .iter()
        .map(|data| NoteTag {
            completed_at: data.completed_at(),
            item_status: data.item_status(),
            definition: data
                .definition()
                .map(|definition_id| parse_note_tag_definition(definition_id, space)),
        })
        .collect()
}

pub(crate) fn parse_note_tag_definition(
    definition_id: ExGuid,
    space: &ObjectSpace,
) -> NoteTagDefinition {
    let object = space
        .get_object(definition_id)
        .expect("note tag definition is missing");

    let data = note_tag_shared_definition_container::parse(object);

    NoteTagDefinition {
        label: data.label().to_string(),
        status: data.status().clone(),
        shape: data.shape(),
        highlight_color: data.highlight_color(),
        text_color: data.text_color(),
        action_item_type: data.action_item_type(),
    }
}
