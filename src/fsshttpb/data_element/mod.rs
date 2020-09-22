use std::fmt::Debug;

use crate::fsshttpb::data_element::object_group::ObjectGroup;
use crate::fsshttpb::data_element::revision_manifest::RevisionManifest;
use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::data_element::value::DataElementValue;
use crate::types::compact_u64::CompactU64;
use crate::types::exguid::ExGuid;
use crate::types::object_types::ObjectType;
use crate::types::serial_number::SerialNumber;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;

pub mod cell_manifest;
pub mod data_element_fragment;
pub mod object_data_blob;
pub mod object_group;
pub mod revision_manifest;
pub mod storage_index;
pub mod storage_manifest;
pub mod value;

#[derive(Debug)]
pub(crate) struct DataElementPackage {
    pub(crate) header: ObjectHeader,
    pub(crate) elements: Vec<DataElement>,
}

impl DataElementPackage {
    pub(crate) fn parse(reader: Reader) -> DataElementPackage {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, ObjectType::DataElementPackage);

        assert_eq!(reader.get_u8(), 0);

        let mut elements = vec![];

        loop {
            if ObjectHeader::try_parse_end_8(reader, ObjectType::DataElementPackage).is_some() {
                break;
            }

            elements.push(DataElement::parse(reader));
        }

        DataElementPackage { header, elements }
    }

    pub(crate) fn find_objects(
        &self,
        cell: ExGuid,
        storage_index: &StorageIndex,
    ) -> Vec<&ObjectGroup> {
        let revision_id = self
            .find_cell_revision_id(cell)
            .expect("cell revision id not found");
        let revision_mapping_id = storage_index
            .find_revision_mapping_id(revision_id)
            .expect("revision mapping id not found");
        let revision_manifest = self
            .find_revision_manifest(revision_mapping_id)
            .expect("revision manifest not found");

        revision_manifest
            .group_references
            .iter()
            .map(|reference| {
                self.find_object_group(*reference)
                    .expect("object group not found")
            })
            .collect()
    }

    pub(crate) fn find_blob(&self, id: ExGuid) -> Option<Vec<u8>> {
        self.find_element(id).map(|element| {
            if let DataElementValue::ObjectDataBlob(data) = &element.element {
                data.value().to_vec()
            } else {
                panic!("data element is not a blob")
            }
        })
    }

    pub(crate) fn find_cell_revision_id(&self, id: ExGuid) -> Option<ExGuid> {
        self.find_element(id).map(|element| {
            if let DataElementValue::CellManifest(revision_id) = &element.element {
                *revision_id
            } else {
                panic!("data element is not a cell manifest")
            }
        })
    }

    pub(crate) fn find_revision_manifest(&self, id: ExGuid) -> Option<&RevisionManifest> {
        self.find_element(id).map(|element| {
            if let DataElementValue::RevisionManifest(revision_manifest) = &element.element {
                revision_manifest
            } else {
                panic!("data element is not a revision manifest")
            }
        })
    }

    pub(crate) fn find_object_group(&self, id: ExGuid) -> Option<&ObjectGroup> {
        self.find_element(id).map(|element| {
            if let DataElementValue::ObjectGroup(object_group) = &element.element {
                object_group
            } else {
                panic!("data element is not an object group")
            }
        })
    }

    fn find_element(&self, id: ExGuid) -> Option<&DataElement> {
        self.elements.iter().find(|element| element.id == id)
    }
}

#[derive(Debug)]
pub(crate) struct DataElement {
    pub(crate) id: ExGuid,
    pub(crate) serial: SerialNumber,
    pub(crate) element: DataElementValue,
}

impl DataElement {
    pub(crate) fn parse(reader: Reader) -> DataElement {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, ObjectType::DataElement);

        let id = ExGuid::parse(reader);
        let serial = SerialNumber::parse(reader);
        let element_type = CompactU64::parse(reader);

        let element = DataElementValue::parse(element_type.value(), reader);

        DataElement {
            id,
            serial,
            element,
        }
    }
}
