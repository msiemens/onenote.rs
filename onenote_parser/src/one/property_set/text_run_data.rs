use crate::errors::{ErrorKind, Result};
use crate::one::property::PropertyType;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::object_space_reference::ObjectSpaceReference;
use crate::onestore::object::Object;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::jcid::JcId;
use crate::onestore::types::object_prop_set::ObjectPropSet;
use crate::onestore::types::prop_set::PropertySet;
use crate::onestore::types::property::PropertyId;

pub(crate) fn parse<'a>(object: &'a Object) -> Result<Option<Vec<Object<'a>>>> {
    let (prop_id, prop_sets) = match object.props().get(PropertyType::TextRunData) {
        Some(value) => value.to_property_values().ok_or_else(|| {
            ErrorKind::MalformedOneNoteFileData(
                "text run data is not a property values list".into(),
            )
        })?,
        None => return Ok(None),
    };

    let data = prop_sets
        .iter()
        .map(|props| parse_object(object, prop_id, props))
        .collect::<Result<Vec<_>>>()?;

    Ok(Some(data))
}

fn parse_object<'a>(
    object: &'a Object,
    prop_id: PropertyId,
    props: &PropertySet,
) -> Result<Object<'a>> {
    Ok(Object {
        context_id: object.context_id,
        jc_id: JcId(prop_id.value()),
        props: ObjectPropSet {
            object_ids: get_object_ids(props, object)?,
            object_space_ids: get_object_space_ids(props, object)?,
            context_ids: vec![],
            properties: props.clone(),
        },
        file_data: None,
        mapping: object.mapping.clone(),
    })
}

fn get_object_ids(props: &PropertySet, object: &Object) -> Result<Vec<CompactId>> {
    Ok(object
        .props
        .object_ids
        .iter()
        .skip(ObjectReference::get_offset(
            PropertyType::TextRunData,
            object,
        )?)
        .take(ObjectReference::count_references(props.values()))
        .copied()
        .collect())
}

fn get_object_space_ids(props: &PropertySet, object: &Object) -> Result<Vec<CompactId>> {
    Ok(object
        .props
        .object_ids
        .iter()
        .skip(ObjectSpaceReference::get_offset(
            PropertyType::TextRunData,
            object,
        )?)
        .take(ObjectSpaceReference::count_references(props.values()))
        .copied()
        .collect())
}
