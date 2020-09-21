use crate::one::property::time::Time;
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct NoteTagState {
    created_at: Time,
    completed_at: Time,
    item_status: ActionItemStatus,
    item_type: ActionItemType,
}

#[derive(Debug)]
pub(crate) struct ActionItemStatus {
    completed: bool,
    disabled: bool,
    task_tag: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum ActionItemType {
    Numeric(u16),
    DueToday,
    DueTomorrow,
    DueThisWeek,
    DueNextWeek,
    NoDueDate,
    CustomDueDate,
}

impl ActionItemType {
    pub(crate) fn parse(object: &Object) -> Option<ActionItemType> {
        object
            .props()
            .get(PropertyType::ActionItemType)
            .map(|value| value.to_u16().expect("action item type is no u16"))
            .map(|value| match value {
                0..=99 => ActionItemType::Numeric(value),
                100 => ActionItemType::DueToday,
                101 => ActionItemType::DueTomorrow,
                102 => ActionItemType::DueThisWeek,
                103 => ActionItemType::DueNextWeek,
                104 => ActionItemType::NoDueDate,
                105 => ActionItemType::CustomDueDate,
                _ => panic!("invalid action item type: {}", value),
            })
    }
}
