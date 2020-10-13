use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::onestore::object::Object;

#[derive(Debug, Copy, Clone)]
pub struct ActionItemStatus {
    completed: bool,
    disabled: bool,
    task_tag: bool,
}

impl ActionItemStatus {
    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn task_tag(&self) -> bool {
        self.task_tag
    }
}

impl ActionItemStatus {
    pub(crate) fn parse(object: &Object) -> Result<Option<ActionItemStatus>> {
        let status = object
            .props()
            .get(PropertyType::ActionItemStatus)
            .map(|value| {
                value.to_u16().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("action item status is not a u16".into())
                })
            })
            .transpose()?
            .map(|value| ActionItemStatus {
                completed: value & 0x1 != 0,
                disabled: (value >> 1) & 0x1 != 0,
                task_tag: (value >> 2) & 0x1 != 0,
            });

        Ok(status)
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum ActionItemType {
    Numeric(u16),
    DueToday,
    DueTomorrow,
    DueThisWeek,
    DueNextWeek,
    NoDueDate,
    CustomDueDate,
    Unknown,
}

impl ActionItemType {
    pub(crate) fn parse(object: &Object) -> Result<Option<ActionItemType>> {
        let item_type = object
            .props()
            .get(PropertyType::ActionItemType)
            .map(|value| {
                value.to_u16().ok_or_else(|| {
                    ErrorKind::MalformedOneNoteFileData("action item type is no u16".into())
                })
            })
            .transpose()?
            .map(|value| match value {
                0..=99 => ActionItemType::Numeric(value),
                100 => ActionItemType::DueToday,
                101 => ActionItemType::DueTomorrow,
                102 => ActionItemType::DueThisWeek,
                103 => ActionItemType::DueNextWeek,
                104 => ActionItemType::NoDueDate,
                105 => ActionItemType::CustomDueDate,
                _ => ActionItemType::Unknown,
            });

        Ok(item_type)
    }
}
