use crate::fsshttpb::data_element::data_element_fragment::DataElementFragment;
use crate::fsshttpb::data_element::object_data_blob::ObjectDataBlob;
use crate::fsshttpb::data_element::object_group::ObjectGroup;
use crate::fsshttpb::data_element::revision_manifest::RevisionManifest;
use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::data_element::storage_manifest::StorageManifest;
use crate::types::compact_u64::CompactU64;
use crate::types::exguid::ExGuid;
use crate::types::object_types::ObjectType;
use crate::types::serial_number::SerialNumber;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;
use std::collections::HashMap;
use std::fmt::Debug;

pub(crate) mod cell_manifest;
pub(crate) mod data_element_fragment;
pub(crate) mod object_data_blob;
pub(crate) mod object_group;
pub(crate) mod revision_manifest;
pub(crate) mod storage_index;
pub(crate) mod storage_manifest;

#[derive(Debug)]
pub(crate) struct DataElementPackage {
    pub(crate) header: ObjectHeader,
    pub(crate) storage_indexes: HashMap<ExGuid, StorageIndex>,
    pub(crate) storage_manifests: HashMap<ExGuid, StorageManifest>,
    pub(crate) cell_manifests: HashMap<ExGuid, ExGuid>,
    pub(crate) revision_manifests: HashMap<ExGuid, RevisionManifest>,
    pub(crate) object_groups: HashMap<ExGuid, ObjectGroup>,
    pub(crate) data_element_fragments: HashMap<ExGuid, DataElementFragment>,
    pub(crate) object_data_blobs: HashMap<ExGuid, ObjectDataBlob>,
}

impl DataElementPackage {
    pub(crate) fn parse(reader: Reader) -> DataElementPackage {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, ObjectType::DataElementPackage);

        assert_eq!(reader.get_u8(), 0);

        let mut package = DataElementPackage {
            header,
            storage_indexes: Default::default(),
            storage_manifests: Default::default(),
            cell_manifests: Default::default(),
            revision_manifests: Default::default(),
            object_groups: Default::default(),
            data_element_fragments: Default::default(),
            object_data_blobs: Default::default(),
        };

        loop {
            if ObjectHeader::try_parse_end_8(reader, ObjectType::DataElementPackage).is_some() {
                break;
            }

            DataElement::parse(reader, &mut package)
        }

        package
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

    pub(crate) fn find_blob(&self, id: ExGuid) -> Option<&[u8]> {
        self.object_data_blobs.get(&id).map(|blob| blob.value())
    }

    pub(crate) fn find_cell_revision_id(&self, id: ExGuid) -> Option<ExGuid> {
        self.cell_manifests.get(&id).copied()
    }

    pub(crate) fn find_revision_manifest(&self, id: ExGuid) -> Option<&RevisionManifest> {
        self.revision_manifests.get(&id)
    }

    pub(crate) fn find_object_group(&self, id: ExGuid) -> Option<&ObjectGroup> {
        self.object_groups.get(&id)
    }
}

#[derive(Debug)]
pub(crate) struct DataElement;

impl DataElement {
    pub(crate) fn parse(reader: Reader, package: &mut DataElementPackage) {
        let header = ObjectHeader::parse_16(reader);
        assert_eq!(header.object_type, ObjectType::DataElement);

        let id = ExGuid::parse(reader);
        let _serial = SerialNumber::parse(reader);
        let element_type = CompactU64::parse(reader);

        match element_type.value() {
            0x01 => {
                package
                    .storage_indexes
                    .insert(id, Self::parse_storage_index(reader));
            }
            0x02 => {
                package
                    .storage_manifests
                    .insert(id, Self::parse_storage_manifest(reader));
            }
            0x03 => {
                package
                    .cell_manifests
                    .insert(id, Self::parse_cell_manifest(reader));
            }
            0x04 => {
                package
                    .revision_manifests
                    .insert(id, Self::parse_revision_manifest(reader));
            }
            0x05 => {
                package
                    .object_groups
                    .insert(id, Self::parse_object_group(reader));
            }
            0x06 => {
                package
                    .data_element_fragments
                    .insert(id, Self::parse_data_element_fragment(reader));
            }
            0x0A => {
                package
                    .object_data_blobs
                    .insert(id, Self::parse_object_data_blob(reader));
            }
            x => panic!("invalid element type: 0x{:X}", x),
        }
    }
}
