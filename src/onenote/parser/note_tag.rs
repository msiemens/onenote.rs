use crate::errors::{ErrorKind, Result};
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
pub struct NoteTag {
    completed_at: Option<Time>,
    item_status: ActionItemStatus,
    definition: Option<NoteTagDefinition>,
}

impl NoteTag {
    pub fn completed_at(&self) -> Option<Time> {
        self.completed_at
    }

    pub fn item_status(&self) -> ActionItemStatus {
        self.item_status
    }

    pub fn definition(&self) -> Option<&NoteTagDefinition> {
        self.definition.as_ref()
    }
}

#[derive(Debug)]
pub struct NoteTagDefinition {
    label: String,
    status: NoteTagPropertyStatus,
    shape: NoteTagShape,
    highlight_color: Option<ColorRef>,
    text_color: Option<ColorRef>,
    action_item_type: ActionItemType,
}

impl NoteTagDefinition {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn status(&self) -> &NoteTagPropertyStatus {
        &self.status
    }

    pub fn shape(&self) -> NoteTagShape {
        self.shape
    }

    pub fn highlight_color(&self) -> Option<ColorRef> {
        self.highlight_color
    }

    pub fn text_color(&self) -> Option<ColorRef> {
        self.text_color
    }

    pub fn action_item_type(&self) -> ActionItemType {
        self.action_item_type
    }
}

pub(crate) fn parse_note_tags(note_tags: Vec<Data>, space: &ObjectSpace) -> Result<Vec<NoteTag>> {
    note_tags
        .into_iter()
        .map(|data| {
            Ok(NoteTag {
                completed_at: data.completed_at,
                item_status: data.item_status,
                definition: data
                    .definition
                    .map(|definition_id| parse_note_tag_definition(definition_id, space))
                    .transpose()?,
            })
        })
        .collect()
}

pub(crate) fn parse_note_tag_definition(
    definition_id: ExGuid,
    space: &ObjectSpace,
) -> Result<NoteTagDefinition> {
    let object = space
        .get_object(definition_id)
        .ok_or_else(|| ErrorKind::MalformedOneNoteData("note tag definition is missing".into()))?;

    let data = note_tag_shared_definition_container::parse(object)?;

    let definition = NoteTagDefinition {
        label: data.label,
        status: data.status,
        shape: data.shape,
        highlight_color: data.highlight_color,
        text_color: data.text_color,
        action_item_type: data.action_item_type,
    };

    Ok(definition)
}
