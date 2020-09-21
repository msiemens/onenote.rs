use crate::fsshttpb::data_element::object_group::ObjectGroupData;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::mapping_table::MappingTable;
use crate::onestore::revision::GroupData;
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

#[derive(Debug, Copy, Clone)]
enum Partition {
    Metadata = 4,
    ObjectData = 1,
    FileData = 2,
}

impl Object {
    pub(crate) fn id(&self) -> JcId {
        self.jc_id
    }

    pub(crate) fn props(&self) -> &ObjectPropSet {
        &self.props
    }

    pub(crate) fn file_data(&self) -> Option<&[u8]> {
        self.file_data.as_deref()
    }

    pub(crate) fn mapping(&self) -> &MappingTable {
        &self.mapping
    }
}

impl Object {
    pub(crate) fn parse(
        object_id: ExGuid,
        object_space_id: ExGuid,
        objects: &GroupData,
        packaging: &Packaging,
    ) -> Object {
        let metadata_object = Object::find_object(object_id, Partition::Metadata, objects)
            .expect("object metadata is missing");
        let data_object = Object::find_object(object_id, Partition::ObjectData, objects)
            .expect("object data is missing");

        // Parse metadata

        let metadata = if let ObjectGroupData::Object { data, .. } = metadata_object {
            data
        } else {
            panic!("object metadata it not an object")
        };

        let jc_id = JcId::parse(&mut metadata.as_slice());

        // Parse data

        let (data, object_refs, referenced_cells) =
            if let ObjectGroupData::Object { group, cells, data } = data_object {
                (data, group, cells)
            } else {
                panic!("object data it not an object")
            };

        let props = ObjectPropSet::parse(&mut data.as_slice());

        // Parse file data

        let file_data = Object::find_blob_id(object_id, objects).map(|blob_id| {
            packaging
                .data_element_package
                .find_blob(blob_id)
                .expect("blob not found")
        });

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

        assert_eq!(props.object_ids().len(), object_refs.len());
        assert_eq!(
            props.context_ids().len() + props.object_space_ids().len(),
            referenced_cells.len()
        );

        let mapping_objects = props
            .object_ids()
            .iter()
            .copied()
            .zip(object_refs.iter().copied());

        let mapping_contexts = props.context_ids().iter().copied().zip(context_refs);

        let mapping_object_spaces = props
            .object_space_ids()
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
        partition_id: Partition,
        objects: &'a GroupData,
    ) -> Option<&'a ObjectGroupData> {
        objects.get(&(id, partition_id as u64)).cloned()
    }

    fn find_blob_id(id: ExGuid, objects: &GroupData) -> Option<ExGuid> {
        Self::find_object(id, Partition::FileData, objects).map(|object| match object {
            ObjectGroupData::BlobReference { blob, .. } => *blob,
            _ => panic!("blob object is not a blob"),
        })
    }
}
