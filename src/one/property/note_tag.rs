use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub(crate) struct ActionItemStatus {
    completed: bool,
    disabled: bool,
    task_tag: bool,
}

impl ActionItemStatus {
    pub(crate) fn parse(object: &Object) -> Option<ActionItemStatus> {
        object
            .props()
            .get(PropertyType::ActionItemStatus)
            .map(|value| value.to_u16().expect("action item status is not a u16"))
            .map(|value| ActionItemStatus {
                completed: value & 0x1 != 0,
                disabled: (value >> 1) & 0x1 != 0,
                task_tag: (value >> 2) & 0x1 != 0,
            })
    }
}

#[derive(Debug, Copy, Clone)]
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
