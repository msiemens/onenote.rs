use crate::one::property::note_tag::ActionItemStatus;
use crate::one::property::object_reference::ObjectReference;
use crate::one::property::object_space_reference::ObjectSpaceReference;
use crate::one::property::time::Time;
use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::onestore::types::compact_id::CompactId;
use crate::onestore::types::jcid::JcId;
use crate::onestore::types::object_prop_set::ObjectPropSet;
use crate::onestore::types::prop_set::PropertySet;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) definition: Option<ExGuid>,
    pub(crate) created_at: Time,
    pub(crate) completed_at: Option<Time>,
    pub(crate) item_status: ActionItemStatus,
}

impl Data {
    pub(crate) fn parse(object: &Object) -> Option<Vec<Data>> {
        object
            .props()
            .get(PropertyType::NoteTags)
            .map(|value| {
                value
                    .to_property_values()
                    .expect("note tag state is not a property values list")
            })
            .map(|(id, sets)| {
                sets.iter()
                    .map(|props| Object {
                        jc_id: JcId(id.value()),
                        props: ObjectPropSet {
                            object_ids: Self::get_object_ids(props, object),
                            object_space_ids: Self::get_object_space_ids(props, object),
                            context_ids: vec![],
                            properties: props.clone(),
                        },
                        file_data: None,
                        mapping: object.mapping.clone(),
                    })
                    .map(|object| Data {
                        definition: ObjectReference::parse(
                            PropertyType::NoteTagDefinitionOid,
                            &object,
                        ),
                        created_at: Time::parse(PropertyType::NoteTagCreated, &object)
                            .expect("note tag has no created at time"),
                        completed_at: Time::parse(PropertyType::NoteTagCompleted, &object),
                        item_status: ActionItemStatus::parse(&object)
                            .expect("note tag container has no item status"),
                    })
                    .collect()
            })
    }

    fn get_object_ids(props: &PropertySet, object: &Object) -> Vec<CompactId> {
        object
            .props
            .object_ids
            .iter()
            .skip(ObjectReference::get_offset(PropertyType::NoteTags, object))
            .take(ObjectReference::count_references(props.values()))
            .copied()
            .collect()
    }

    fn get_object_space_ids(props: &PropertySet, object: &Object) -> Vec<CompactId> {
        object
            .props
            .object_ids
            .iter()
            .skip(ObjectSpaceReference::get_offset(
                PropertyType::NoteTags,
                object,
            ))
            .take(ObjectSpaceReference::count_references(props.values()))
            .copied()
            .collect()
    }
}
