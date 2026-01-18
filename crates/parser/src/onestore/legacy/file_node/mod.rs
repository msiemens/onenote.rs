pub(crate) mod file_node_chunk_reference;
pub(crate) mod global_id_table;
pub(crate) mod object_declaration;
pub(crate) mod object_revision;
pub(crate) mod object_space_manifest;
pub(crate) mod revision_manifest;
pub(crate) mod root_object_reference;
pub(crate) mod shared;

use crate::Reader;
use crate::errors::ErrorKind;
use crate::onestore::legacy::common::FileChunkReference;
use crate::onestore::legacy::file_node::file_node_chunk_reference::FileNodeChunkReference;
use crate::onestore::legacy::file_node::global_id_table::{
    GlobalIdTableEntry2FNDX, GlobalIdTableEntry3FNDX, GlobalIdTableEntryFNDX,
    GlobalIdTableStartFNDX,
};
use crate::onestore::legacy::file_node::object_declaration::{
    ObjectDeclaration2LargeRefCountFND, ObjectDeclaration2RefCountFND,
    ObjectDeclarationFileData3LargeRefCountFND, ObjectDeclarationFileData3RefCountFND,
    ObjectDeclarationWithRefCount2FNDX, ObjectDeclarationWithRefCountFNDX,
    ReadOnlyObjectDeclaration2LargeRefCountFND, ReadOnlyObjectDeclaration2RefCountFND,
};
use crate::onestore::legacy::file_node::object_revision::{
    ObjectRevisionWithRefCount2FNDX, ObjectRevisionWithRefCountFNDX,
};
use crate::onestore::legacy::file_node::object_space_manifest::{
    ObjectSpaceManifestListReferenceFND, ObjectSpaceManifestListStartFND,
    ObjectSpaceManifestRootFND,
};
use crate::onestore::legacy::file_node::revision_manifest::{
    RevisionManifestListReferenceFND, RevisionManifestListStartFND, RevisionManifestStart4FND,
    RevisionManifestStart6FND, RevisionManifestStart7FND,
};
use crate::onestore::legacy::file_node::root_object_reference::{
    RootObjectReference2FNDX, RootObjectReference3FND,
};
use crate::onestore::legacy::file_node::shared::{
    DataSignatureGroupDefinitionFND, FileDataStoreListReferenceFND,
    FileDataStoreObjectReferenceFND, HashedChunkDescriptor2FND, ObjectDataEncryptionKeyV2FNDX,
    ObjectGroupListReferenceFND, ObjectGroupStartFND, ObjectInfoDependencyOverridesFND,
    ParseWithRef, RevisionRoleAndContextDeclarationFND, RevisionRoleDeclarationFND, UnknownNode,
};
use crate::onestore::legacy::file_structure::{FileNodeList, ParseContext};
use crate::onestore::legacy::parse::{Parse, ParseWithCount};

/// See [\[MS-ONESTORE\] 2.4.3](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/25a9b048-f91a-48d1-b803-137b7194e69e)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct FileNode {
    /// Specifies the type of the structure
    node_type_id: u32,

    stp_format: u32,
    cb_format: u32,
    base_type: u32,
    pub(crate) size: usize,
    pub(crate) fnd: FileNodeData,
}

#[derive(Debug, Clone)]
pub(crate) enum FileNodeDataRef {
    SingleElement(FileNodeChunkReference),
    ElementList(FileNodeList),
    NoData,
    InvalidData,
}

/// See [\[MS-ONESTORE\] 2.4.3](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/25a9b048-f91a-48d1-b803-137b7194e69e)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum FileNodeData {
    ObjectSpaceManifestRootFND(ObjectSpaceManifestRootFND),
    ObjectSpaceManifestListReferenceFND(ObjectSpaceManifestListReferenceFND),
    ObjectSpaceManifestListStartFND(ObjectSpaceManifestListStartFND),
    RevisionManifestListReferenceFND(RevisionManifestListReferenceFND),
    RevisionManifestListStartFND(RevisionManifestListStartFND),
    RevisionManifestStart4FND(RevisionManifestStart4FND),
    RevisionManifestEndFND,
    RevisionManifestStart6FND(RevisionManifestStart6FND),
    RevisionManifestStart7FND(RevisionManifestStart7FND),
    GlobalIdTableStartFNDX(GlobalIdTableStartFNDX),
    GlobalIdTableStart2FND,
    GlobalIdTableEntryFNDX(GlobalIdTableEntryFNDX),
    GlobalIdTableEntry2FNDX(GlobalIdTableEntry2FNDX),
    GlobalIdTableEntry3FNDX(GlobalIdTableEntry3FNDX),
    GlobalIdTableEndFNDX,
    ObjectDeclarationWithRefCountFNDX(ObjectDeclarationWithRefCountFNDX),
    ObjectDeclarationWithRefCount2FNDX(ObjectDeclarationWithRefCount2FNDX),
    ObjectRevisionWithRefCountFNDX(ObjectRevisionWithRefCountFNDX),
    ObjectRevisionWithRefCount2FNDX(ObjectRevisionWithRefCount2FNDX),
    RootObjectReference2FNDX(RootObjectReference2FNDX),
    RootObjectReference3FND(RootObjectReference3FND),
    RevisionRoleDeclarationFND(RevisionRoleDeclarationFND),
    RevisionRoleAndContextDeclarationFND(RevisionRoleAndContextDeclarationFND),
    ObjectDeclarationFileData3RefCountFND(ObjectDeclarationFileData3RefCountFND),
    ObjectDeclarationFileData3LargeRefCountFND(ObjectDeclarationFileData3LargeRefCountFND),
    ObjectDataEncryptionKeyV2FNDX(ObjectDataEncryptionKeyV2FNDX),
    ObjectInfoDependencyOverridesFND(ObjectInfoDependencyOverridesFND),
    DataSignatureGroupDefinitionFND(DataSignatureGroupDefinitionFND),
    FileDataStoreListReferenceFND(FileDataStoreListReferenceFND),
    FileDataStoreObjectReferenceFND(FileDataStoreObjectReferenceFND),
    ObjectDeclaration2RefCountFND(ObjectDeclaration2RefCountFND),
    ObjectDeclaration2LargeRefCountFND(ObjectDeclaration2LargeRefCountFND),
    ObjectGroupListReferenceFND(ObjectGroupListReferenceFND),
    ObjectGroupStartFND(ObjectGroupStartFND),
    ObjectGroupEndFND,
    HashedChunkDescriptor2FND(HashedChunkDescriptor2FND),
    ReadOnlyObjectDeclaration2RefCountFND(ReadOnlyObjectDeclaration2RefCountFND),
    ReadOnlyObjectDeclaration2LargeRefCountFND(ReadOnlyObjectDeclaration2LargeRefCountFND),
    ChunkTerminatorFND,
    UnknownNode(UnknownNode),
    Null,
}

impl FileNode {
    pub(crate) fn parse(reader: Reader, context: &mut ParseContext) -> crate::errors::Result<Self> {
        let remaining_0 = reader.remaining();
        let first_line = reader.get_u32()?;
        let node_id = first_line & 0x3FF; // First 10 bits
        let size = ((first_line >> 10) & 0x1FFF) as usize; // Next 13 bits
        let stp_format = (first_line >> 23) & 0x3; // Next 2 bits
        let cb_format = (first_line >> 25) & 0x3; // Next 2 bits
        let base_type = (first_line >> 27) & 0xF;
        // Last bit is reserved

        let data_ref: FileNodeDataRef = match base_type {
            0 => FileNodeDataRef::NoData, // Does not reference other data
            1 => FileNodeDataRef::SingleElement(FileNodeChunkReference::parse(
                reader, stp_format, cb_format,
            )?),
            2 => FileNodeDataRef::ElementList({
                let list_ref = FileNodeChunkReference::parse(reader, stp_format, cb_format)?;
                let mut resolved_reader = list_ref.resolve_to_reader(reader)?;
                FileNodeList::parse(&mut resolved_reader, context, list_ref.data_size())
            }?),
            _ => FileNodeDataRef::InvalidData,
        };

        let remaining_1 = reader.remaining();

        let fnd = match node_id {
            0x004 => {
                FileNodeData::ObjectSpaceManifestRootFND(ObjectSpaceManifestRootFND::parse(reader)?)
            }
            0x008 => FileNodeData::ObjectSpaceManifestListReferenceFND(
                ObjectSpaceManifestListReferenceFND::parse(reader, &data_ref)?,
            ),
            0x00C => FileNodeData::ObjectSpaceManifestListStartFND(
                ObjectSpaceManifestListStartFND::parse(reader)?,
            ),
            0x010 => FileNodeData::RevisionManifestListReferenceFND(
                RevisionManifestListReferenceFND::parse(reader, &data_ref)?,
            ),
            0x014 => FileNodeData::RevisionManifestListStartFND(
                RevisionManifestListStartFND::parse(reader)?,
            ),
            0x01B => {
                FileNodeData::RevisionManifestStart4FND(RevisionManifestStart4FND::parse(reader)?)
            }
            0x01C => FileNodeData::RevisionManifestEndFND,
            0x01E => {
                FileNodeData::RevisionManifestStart6FND(RevisionManifestStart6FND::parse(reader)?)
            }
            0x01F => {
                FileNodeData::RevisionManifestStart7FND(RevisionManifestStart7FND::parse(reader)?)
            }
            0x021 => FileNodeData::GlobalIdTableStartFNDX(GlobalIdTableStartFNDX::parse(reader)?),
            0x022 => FileNodeData::GlobalIdTableStart2FND,
            0x024 => FileNodeData::GlobalIdTableEntryFNDX(GlobalIdTableEntryFNDX::parse(reader)?),
            0x025 => FileNodeData::GlobalIdTableEntry2FNDX(GlobalIdTableEntry2FNDX::parse(reader)?),
            0x026 => FileNodeData::GlobalIdTableEntry3FNDX(GlobalIdTableEntry3FNDX::parse(reader)?),
            0x028 => FileNodeData::GlobalIdTableEndFNDX,
            0x02D => FileNodeData::ObjectDeclarationWithRefCountFNDX(
                ObjectDeclarationWithRefCountFNDX::parse(reader, &data_ref)?,
            ),
            0x02E => FileNodeData::ObjectDeclarationWithRefCount2FNDX(
                ObjectDeclarationWithRefCount2FNDX::parse(reader, &data_ref)?,
            ),
            0x041 => FileNodeData::ObjectRevisionWithRefCountFNDX(
                ObjectRevisionWithRefCountFNDX::parse(reader, &data_ref)?,
            ),
            0x042 => FileNodeData::ObjectRevisionWithRefCount2FNDX(
                ObjectRevisionWithRefCount2FNDX::parse(reader, &data_ref)?,
            ),
            0x059 => {
                FileNodeData::RootObjectReference2FNDX(RootObjectReference2FNDX::parse(reader)?)
            }
            0x05A => FileNodeData::RootObjectReference3FND(RootObjectReference3FND::parse(reader)?),
            0x05C => {
                FileNodeData::RevisionRoleDeclarationFND(RevisionRoleDeclarationFND::parse(reader)?)
            }
            0x05D => FileNodeData::RevisionRoleAndContextDeclarationFND(
                RevisionRoleAndContextDeclarationFND::parse(reader)?,
            ),
            0x072 => FileNodeData::ObjectDeclarationFileData3RefCountFND(
                ObjectDeclarationFileData3RefCountFND::parse(reader)?,
            ),
            0x073 => FileNodeData::ObjectDeclarationFileData3LargeRefCountFND(
                ObjectDeclarationFileData3LargeRefCountFND::parse(reader)?,
            ),
            0x07C => FileNodeData::ObjectDataEncryptionKeyV2FNDX(
                ObjectDataEncryptionKeyV2FNDX::parse(reader)?,
            ),
            0x084 => FileNodeData::ObjectInfoDependencyOverridesFND(
                ObjectInfoDependencyOverridesFND::parse(reader, &data_ref)?,
            ),
            0x08C => FileNodeData::DataSignatureGroupDefinitionFND(
                DataSignatureGroupDefinitionFND::parse(reader)?,
            ),
            0x090 => FileNodeData::FileDataStoreListReferenceFND(
                FileDataStoreListReferenceFND::parse(reader, &data_ref)?,
            ),
            0x094 => FileNodeData::FileDataStoreObjectReferenceFND(
                FileDataStoreObjectReferenceFND::parse(reader, &data_ref)?,
            ),
            0x0A4 => FileNodeData::ObjectDeclaration2RefCountFND(
                ObjectDeclaration2RefCountFND::parse(reader, &data_ref)?,
            ),
            0x0A5 => FileNodeData::ObjectDeclaration2LargeRefCountFND(
                ObjectDeclaration2LargeRefCountFND::parse(reader, &data_ref)?,
            ),
            0x0B0 => FileNodeData::ObjectGroupListReferenceFND(ObjectGroupListReferenceFND::parse(
                reader, &data_ref,
            )?),
            0x0B4 => FileNodeData::ObjectGroupStartFND(ObjectGroupStartFND::parse(reader)?),
            0x0B8 => FileNodeData::ObjectGroupEndFND,
            0x0C2 => FileNodeData::HashedChunkDescriptor2FND(HashedChunkDescriptor2FND::parse(
                reader, &data_ref,
            )?),
            0x0C4 => FileNodeData::ReadOnlyObjectDeclaration2RefCountFND(
                ReadOnlyObjectDeclaration2RefCountFND::parse(reader, &data_ref)?,
            ),
            0x0C5 => FileNodeData::ReadOnlyObjectDeclaration2LargeRefCountFND(
                ReadOnlyObjectDeclaration2LargeRefCountFND::parse(reader, &data_ref)?,
            ),
            0x0FF => FileNodeData::ChunkTerminatorFND,
            0 => FileNodeData::Null,
            other => {
                log::warn!("Unknown node type: {:#0x}, size {}", other, size);
                let size_used = remaining_0 - remaining_1;
                assert!(size_used <= size);
                let remaining_size = size - size_used;
                FileNodeData::UnknownNode(UnknownNode::parse(reader, remaining_size)?)
            }
        };

        let remaining_2 = reader.remaining();
        let actual_size = remaining_0 - remaining_2;

        let node = Self {
            node_type_id: node_id,
            stp_format,
            cb_format,
            base_type,
            size: actual_size,
            fnd,
        };

        // The stored size can be incorrect when node_id is zero
        if actual_size != size && node_id != 0 {
            println!(
                "Incorrect structure size: {:#?} (expected size {}, but was {})",
                node, size, actual_size
            );

            Err(ErrorKind::MalformedOneNoteFileData(
                format!(
                    "The size specified for this structure is incorrect. Was {}, expected {}. Id: {:#0x}",
                    actual_size, size, node_id
                )
                    .into(),
            )
                .into())
        } else {
            Ok(node)
        }
    }
}
