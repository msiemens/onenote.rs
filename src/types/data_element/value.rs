use crate::data::exguid::ExGuid;
use crate::data::guid::Guid;
use crate::types::data_element::data_element_fragment::DataElementFragmentChunkReference;
use crate::types::data_element::object_group::{
    ObjectGroupData, ObjectGroupDeclaration, ObjectGroupMetadata,
};
use crate::types::data_element::revision_manifest::RevisionManifestRootDeclare;
use crate::types::data_element::storage_index::{
    StorageIndexCellMapping, StorageIndexManifestMapping, StorageIndexRevisionMapping,
};
use crate::types::data_element::storage_manifest::StorageManifestRoot;
use crate::Reader;

#[derive(Debug)]
pub(crate) enum DataElementValue {
    StorageIndex {
        manifest_mappings: Vec<StorageIndexManifestMapping>,
        cell_mappings: Vec<StorageIndexCellMapping>,
        revision_mappings: Vec<StorageIndexRevisionMapping>,
    },
    StorageManifest {
        id: Guid,
        roots: Vec<StorageManifestRoot>,
    },
    CellManifest(ExGuid),
    RevisionManifest {
        rev_id: ExGuid,
        base_rev_id: ExGuid,
        root_declare: Vec<RevisionManifestRootDeclare>,
        group_references: Vec<ExGuid>,
    },
    ObjectGroup {
        declarations: Vec<ObjectGroupDeclaration>,
        metadata: Vec<ObjectGroupMetadata>,
        objects: Vec<ObjectGroupData>,
    },
    DataElementFragment {
        id: ExGuid,
        size: u64,
        chunk_reference: DataElementFragmentChunkReference,
        data: Vec<u8>,
    },
    ObjectDataBlob(Vec<u8>),
}

impl DataElementValue {
    pub(crate) fn parse(element_type: u64, reader: Reader) -> DataElementValue {
        match element_type {
            0x01 => Self::parse_storage_index(reader),
            0x02 => Self::parse_storage_manifest(reader),
            0x03 => Self::parse_cell_manifest(reader),
            0x04 => Self::parse_revision_manifest(reader),
            0x05 => Self::parse_object_group(reader),
            0x06 => Self::parse_data_element_fragment(reader),
            0x0A => Self::parse_object_data_blob(reader),
            _ => unimplemented!(),
        }
    }
}
