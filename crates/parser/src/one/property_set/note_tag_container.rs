use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property::PropertyType;
use crate::one::property::note_tag::ActionItemStatus;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property_set::parse_object;
use crate::onestore::Object;

/// A note tag state container.
///
/// See [\[MS-ONE\] 2.2.88].
///
/// [\[MS-ONE\] 2.2.88]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/a9938236-87f8-41b1-81f3-5f760e1247b8
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Data {
    pub(crate) definition: Option<ExGuid>,
    pub(crate) created_at: Time,
    pub(crate) completed_at: Option<Time>,
    pub(crate) item_status: ActionItemStatus,
}

impl Data {
    pub(crate) fn parse(object: &Object) -> Result<Option<Vec<Data>>> {
        let (prop_id, prop_sets) = match object.props.get(PropertyType::NoteTags) {
            Some(value) => value.to_property_values().ok_or_else(|| {
                ErrorKind::MalformedOneNoteFileData(
                    "note tag state is not a property values list".into(),
                )
            })?,
            None => return Ok(None),
        };

        let data = prop_sets
            .iter()
            .map(|props| {
                let object = parse_object(object, prop_id, PropertyType::NoteTags, props)?;
                let data = Self::parse_data(object)?;

                Ok(data)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Some(data))
    }

    fn parse_data(object: Object) -> Result<Data> {
        let definition = ObjectReference::parse(PropertyType::NoteTagDefinitionOid, &object)?;

        let created_at = Time::parse(PropertyType::NoteTagCreated, &object)?.ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("note tag has no created at time".into())
        })?;

        let completed_at = Time::parse(PropertyType::NoteTagCompleted, &object)?;

        let item_status = ActionItemStatus::parse(&object)?.ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData("note tag container has no item status".into())
        })?;

        Ok(Data {
            definition,
            created_at,
            completed_at,
            item_status,
        })
    }
}
