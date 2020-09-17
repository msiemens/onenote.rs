use crate::fsshttpb::data_element::object_group::{ObjectGroupData, ObjectGroupDeclaration};
use crate::onestore::mapping_table::MappingTable;
use crate::onestore::types::jcid::JcId;
use crate::onestore::types::object_prop_set::ObjectPropSet;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Object {
    jc_id: JcId,
    props: ObjectPropSet,
    file_data: Option<Vec<u8>>,
    mapping: MappingTable,
}

impl Object {
    pub(crate) fn parse(
        object_id: ExGuid,
        object_space_id: ExGuid,
        objects: &[(&ObjectGroupDeclaration, &ObjectGroupData)],
    ) -> Object {
        let metadata_object = Object::find_object(object_id, objects, 4);
        let data_object = Object::find_object(object_id, objects, 1);

        // Parse metadata

        let metadata = if let ObjectGroupData::Object { data, .. } = metadata_object {
            data
        } else {
            panic!("object group metadata it not an object")
        };

        let jc_id = JcId::parse(&mut metadata.as_slice());

        // Parse data

        let (data, object_refs, referenced_cells) =
            if let ObjectGroupData::Object { group, cells, data } = data_object {
                (data, group, cells)
            } else {
                panic!("object group metadata it not an object")
            };

        let props = ObjectPropSet::parse(&mut data.as_slice());

        // Parse file data

        let file_data = None;
        if jc_id.is_file_data() {
            // FIXME: Read file data
            unimplemented!()

            // let file_data_group = objects
            //     .iter()
            //     .find(|(decl, _)| decl.object_id() == id && decl.partition_id() == 1)
            //     .map(|(_, obj)| obj)
            //     .expect("object not found");
        }

        let context_refs: Vec<_> = referenced_cells
            .iter()
            .filter(|id| id.1 == object_space_id)
            .map(|id| id.0)
            .collect();

        let object_space_refs: Vec<_> = referenced_cells
            .iter()
            .filter(|id| id.1 != object_space_id)
            .map(|id| id.1)
            .collect();

        assert_eq!(props.object_ids.len(), object_refs.len());
        assert_eq!(
            props.context_ids.len() + props.object_space_ids.len(),
            referenced_cells.len()
        );

        let mapping_objects = props
            .object_ids
            .iter()
            .copied()
            .zip(object_refs.iter().copied());

        let mapping_contexts = props.context_ids.iter().copied().zip(context_refs);

        let mapping_object_spaces = props
            .object_space_ids
            .iter()
            .copied()
            .zip(object_space_refs);

        let mapping = MappingTable::from_entries(
            mapping_objects
                .chain(mapping_contexts)
                .chain(mapping_object_spaces),
        );

        Object {
            jc_id,
            props,
            file_data,
            mapping,
        }
    }

    fn find_object<'a>(
        id: ExGuid,
        objects: &'a [(&'a ObjectGroupDeclaration, &'a ObjectGroupData)],
        partition_id: u64,
    ) -> &'a ObjectGroupData {
        objects
            .iter()
            .find(|(decl, _)| decl.object_id() == id && decl.partition_id() == partition_id)
            .map(|(_, obj)| obj)
            .unwrap_or_else(|| panic!("no object with partition id {} found", partition_id))
    }
}
