use crate::fsshttpb::packaging::Packaging;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct ObjectGroup {
    objects: HashMap<ExGuid, Object>,
}

impl ObjectGroup {
    pub(crate) fn objects(&self) -> &HashMap<ExGuid, Object> {
        &self.objects
    }
}

impl ObjectGroup {
    pub(crate) fn parse(
        group_id: ExGuid,
        object_space_id: ExGuid,
        packaging: &Packaging,
    ) -> ObjectGroup {
        let group = packaging
            .data_element_package
            .find_object_group(group_id)
            .expect("object group not found");

        let mut objects = HashMap::new();
        let object_ids: Vec<_> = group.declarations.iter().map(|o| o.object_id()).collect();

        for object_id in object_ids {
            assert_eq!(group.declarations.len(), group.objects.len());

            let object = Object::parse(
                object_id,
                object_space_id,
                &*group
                    .declarations
                    .iter()
                    .zip(group.objects.iter())
                    .collect::<Vec<_>>(),
                packaging,
            );

            objects.insert(object_id, object);
        }

        ObjectGroup { objects }
    }
}
