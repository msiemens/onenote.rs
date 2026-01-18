use crate::Reader;
use crate::errors::ErrorKind;
use crate::errors::Result;
use crate::onestore::legacy::ExGuid;
use crate::onestore::legacy::common::FileChunkReference;
use crate::onestore::legacy::file_node::FileNodeDataRef;
use crate::onestore::legacy::file_structure::FileNodeList;
use crate::onestore::legacy::parse::{Parse, ParseWithCount};
use crate::onestore::shared::compact_id::CompactId;
use crate::onestore::shared::file_blob::FileBlob;
use crate::onestore::shared::jcid::JcId;
use crate::onestore::shared::object_prop_set::ObjectPropSet;
use crate::shared::guid::Guid;
use crate::utils::Utf16ToString;
use parser_macros::Parse;
use std::fmt::Debug;

pub(crate) trait ParseWithRef<'a>
where
    Self: Sized,
{
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> Result<Self>;
}

pub(crate) fn read_property_set(
    reader: Reader,
    property_set_ref: &FileNodeDataRef,
) -> Result<ObjectPropSet> {
    match property_set_ref {
        FileNodeDataRef::SingleElement(data_ref) => {
            let mut prop_set_reader = data_ref.resolve_to_reader(reader)?;
            let prop_set = ObjectPropSet::parse(&mut prop_set_reader)?;
            Ok(prop_set)
        }
        FileNodeDataRef::ElementList(_) => Err(ErrorKind::MalformedOneStoreData(
            "Expected a single element (reading PropertySet)".into(),
        )
        .into()),
        _ => Err(
            ErrorKind::MalformedOneStoreData("Expected a reference to a property set".into())
                .into(),
        ),
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct PointerToListFND {
    pub(crate) list: FileNodeList,
}

impl<'a> ParseWithRef<'a> for PointerToListFND {
    fn parse(_reader: Reader, data_ref: &FileNodeDataRef) -> Result<Self> {
        match data_ref {
            FileNodeDataRef::ElementList(list) => Ok(Self { list: list.clone() }),
            other => Err(onestore_parse_error!("Expected a list, got {:?}", other).into()),
        }
    }
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RevisionRoleDeclarationFND {
    pub(crate) rid: ExGuid,
    /// "should be 0x01"
    revision_role: u32,
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct RevisionRoleAndContextDeclarationFND {
    /// Revision role & pointer to the revision
    pub(crate) base: RevisionRoleDeclarationFND,
    /// The revision context
    pub(crate) gctxid: ExGuid,
}

/// See [\[MS-ONESTORE\] 2.2.3](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/af15f3eb-f2a8-4333-8d04-e05e55c2af07)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct StringInStorageBuffer {
    cch: usize,
    pub(crate) data: String,
}

impl Parse for StringInStorageBuffer {
    fn parse(reader: Reader) -> Result<Self> {
        let characer_count = reader.get_u32()? as usize;
        let string_size = characer_count * 2; // 2 bytes per character
        let data = reader.read(string_size)?;
        let data = data.utf16_to_string()?;
        Ok(Self {
            cch: characer_count,
            data,
        })
    }
}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct ObjectRefAndId<Id: Parse> {
    id: Id,
}

/// Points to encrypted data. See [\[MS-ONESTORE\] 2.5.19](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/542f09eb-9db8-4b6a-86e5-2d9a930b41c0).
#[derive(Debug, Clone, Parse)]
pub(crate) struct ObjectDataEncryptionKeyV2FNDX {}

#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
struct ObjectInfoDependencyOverride<RefSize: Parse> {
    oid: CompactId,
    c_ref: RefSize,
}

/// See [\[MS-ONESTORE\] 2.6.10](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/af821117-689f-42cf-8136-c72c1e238f1e)
#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
struct ObjectInfoDependencyOverrideData {
    c8_override_count: u32,
    c32_override_count: u32,
    crc: u32,
    #[parse_additional_args(c8_override_count as usize)]
    overrides1: Vec<ObjectInfoDependencyOverride<u8>>,
    #[parse_additional_args(c32_override_count as usize)]
    overrides2: Vec<ObjectInfoDependencyOverride<u32>>,
}

/// See [\[MS-ONESTORE\] 2.5.20](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/80125c83-199e-43b9-9a13-4085752eddac)
/// Specifies reference counts for objects.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ObjectInfoDependencyOverridesFND {
    data: ObjectInfoDependencyOverrideData,
}

impl<'a> ParseWithRef<'a> for ObjectInfoDependencyOverridesFND {
    fn parse(reader: Reader, obj_ref: &FileNodeDataRef) -> Result<Self> {
        if let FileNodeDataRef::SingleElement(obj_ref) = obj_ref {
            if !obj_ref.is_fcr_nil() {
                let data = ObjectInfoDependencyOverrideData::parse(
                    &mut obj_ref.resolve_to_reader(reader)?,
                )?;
                Ok(Self { data })
            } else {
                Ok(Self {
                    data: ObjectInfoDependencyOverrideData::parse(reader)?,
                })
            }
        } else {
            Err(ErrorKind::MalformedOneStoreData(
                "Missing ref to data (parsing ObjectInfoDependencyOverridesFND)".into(),
            )
            .into())
        }
    }
}

/// Terminates ObjectGroupEndFND, DataSignatureGroupDefinitionFND, and RevisionManifestEndFND.
/// See [\[MS-ONESTORE\] 2.5.33](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/0fa4c886-011a-4c19-9651-9a69e43a19c6)
#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct DataSignatureGroupDefinitionFND {
    data_signature_group: ExGuid,
}

/// See [\[MS-ONESTORE\] 2.5.21](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/2701cc42-3601-49f9-a3ba-7c40cd8a2be9)
pub(crate) type FileDataStoreListReferenceFND = PointerToListFND;

/// See [\[MS-ONESTORE\] 2.6.13](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/8806fd18-6735-4874-b111-227b83eaac26)
#[derive(Debug, Parse, Clone)]
#[validate(guid_header == Guid::from_str("{BDE316E7-2665-4511-A4C4-8D4D0B7A9EAC}").unwrap())]
#[validate(guid_footer == Guid::from_str("{71FBA722-0F79-4A0B-BB13-899256426B24}").unwrap())]
#[allow(unused)]
pub(crate) struct FileDataStoreObject {
    guid_header: Guid,
    /// Length of the file data (without padding)
    cb_length: u64,
    _unused: u32,
    _reserved: u64,
    #[parse_additional_args(cb_length as usize)]
    #[pad_to_alignment(8)]
    pub(crate) file_data: FileData,
    guid_footer: Guid,
}

#[derive(Clone)]
pub(crate) struct FileData(pub(crate) FileBlob);

impl Debug for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileData(size={:} KiB)", self.0.as_ref().len() / 1024)
    }
}

impl ParseWithCount for FileData {
    fn parse(reader: Reader, size: usize) -> Result<Self> {
        let data = reader.read(size)?.to_vec();
        Ok(FileData(data.into()))
    }
}

/// See [\[MS-ONESTORE\] 2.5.22](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/6f6d5729-ad03-420f-b8fa-7683751218b3)
#[derive(Debug, Clone)]
pub(crate) struct FileDataStoreObjectReferenceFND {
    pub(crate) target: FileDataStoreObject,
    pub(crate) guid: Guid,
}

impl<'a> ParseWithRef<'a> for FileDataStoreObjectReferenceFND {
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> Result<Self> {
        let guid = Guid::parse(reader)?;
        if let FileNodeDataRef::SingleElement(data_ref) = data_ref {
            let mut reader = data_ref.resolve_to_reader(reader)?;
            Ok(Self {
                target: FileDataStoreObject::parse(&mut reader)?,
                guid,
            })
        } else {
            Err(onestore_parse_error!(
                "FileDataStoreObjectReferenceFND should point to a single file node object"
            )
            .into())
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AttachmentInfo {
    pub(crate) extension: String,
    pub(crate) data_ref: String,
}

impl AttachmentInfo {
    pub(crate) fn load_data<T, F>(&self, file_blob_by_id: F) -> Result<T>
    where
        F: FnOnce(&str) -> Result<T>,
    {
        // See https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/da2bbc7d-0529-4bf4-a843-6f3f55c87e8f
        if self.data_ref.starts_with("<ifndf>") {
            file_blob_by_id(&self.data_ref["<ifndf>".len()..])
        } else if self.data_ref.starts_with("<file>") {
            // An external file reference
            // TODO: Find a test .one file that uses this and implement it.
            Err(parser_error!(
                ResolutionFailed,
                "Not supported: Loading an attachment from a file: {} (ext: {})",
                self.data_ref,
                self.extension,
            )
            .into())
        } else if self.data_ref.starts_with("<invfdo>") {
            // "invalid"
            log_warn!("Attempted to load an invalid {} file", self.extension);
            Err(parser_error!(
                ResolutionFailed,
                "Unable to load invalid file reference: {} (ext: {})",
                self.data_ref,
                self.extension
            )
            .into())
        } else {
            Err(parser_error!(
                ResolutionFailed,
                "Failed to resolve file reference: {} (ext: {})",
                self.data_ref,
                self.extension
            )
            .into())
        }
    }
}

/// Common functionality available for most nodes that declare objects
pub(crate) trait ObjectDeclarationNode {
    fn id(&self) -> JcId;
    fn compact_id(&self) -> CompactId;
    fn props(&self) -> Option<&ObjectPropSet>;
    fn get_attachment_info(&self) -> Option<AttachmentInfo> {
        None
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ObjectDeclaration2Body {
    /// The object ID
    oid: CompactId,
    /// Specifies the object type
    jcid: JcId,
    f_has_oid_references: bool,
    f_has_osid_references: bool,
}

impl Parse for ObjectDeclaration2Body {
    fn parse(reader: Reader) -> Result<Self> {
        let oid = CompactId::parse(reader)?;
        let jcid = JcId::parse(reader)?;
        let metadata = reader.get_u8()?;

        Ok(Self {
            oid,
            jcid,
            f_has_oid_references: metadata & 0x1 > 0,
            f_has_osid_references: metadata & 0x2 > 0,
        })
    }
}

/// See [\[MS-ONESTORE\] 2.5.25](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/a6ea1707-b205-4cd8-be40-d4c3462b226b)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ObjectDeclaration2RefCount<RefSize: Parse> {
    props: ObjectPropSet,
    body: ObjectDeclaration2Body,
    c_ref: RefSize,
}

impl<RefSize: Parse> ObjectDeclarationNode for ObjectDeclaration2RefCount<RefSize> {
    fn id(&self) -> JcId {
        self.body.jcid
    }

    fn compact_id(&self) -> CompactId {
        self.body.oid
    }

    fn props(&self) -> Option<&ObjectPropSet> {
        Some(&self.props)
    }
}

impl<'a, RefSize: Parse> ParseWithRef<'a> for ObjectDeclaration2RefCount<RefSize> {
    fn parse(reader: Reader, property_set_ref: &FileNodeDataRef) -> Result<Self> {
        Ok(Self {
            props: read_property_set(reader, property_set_ref)?,
            body: ObjectDeclaration2Body::parse(reader)?,
            c_ref: RefSize::parse(reader)?,
        })
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct ObjectGroupListReferenceFND {
    pub(crate) list: FileNodeList,
    pub(crate) id: ExGuid,
}

impl<'a> ParseWithRef<'a> for ObjectGroupListReferenceFND {
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> Result<Self> {
        match data_ref {
            FileNodeDataRef::ElementList(list) => Ok(Self {
                list: list.clone(),
                id: ExGuid::parse(reader)?,
            }),
            other => Err(parser_error!(
                MalformedOneStoreData,
                "Expected a list (parsing ObjectGroupListReferenceFND), got {:?}",
                other
            )
            .into()),
        }
    }
}

/// See [MS-ONESTORE 2.5.32](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/2b639cb8-1185-4f63-82cb-0f3e4106611e)
#[derive(Debug, Clone, Parse)]
#[allow(dead_code)]
pub(crate) struct ObjectGroupStartFND {
    /// The ID of the object group
    pub(crate) oid: ExGuid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct HashedChunkDescriptor<Hash: Parse> {
    prop_set: ObjectPropSet,
    hash: Hash,
}

impl<'a, Hash: Parse> ParseWithRef<'a> for HashedChunkDescriptor<Hash> {
    fn parse(reader: Reader, prop_ref: &FileNodeDataRef) -> Result<Self> {
        let prop_set = read_property_set(reader, prop_ref)?;
        Ok(Self {
            prop_set,
            hash: Hash::parse(reader)?,
        })
    }
}

pub(crate) type HashedChunkDescriptor2FND = HashedChunkDescriptor<u128>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ReadOnlyObjectDeclaration2RefCount<Base> {
    pub(crate) base: Base,
    md5_hash: u128,
}

impl<'a, Base: ParseWithRef<'a>> ParseWithRef<'a> for ReadOnlyObjectDeclaration2RefCount<Base> {
    fn parse(reader: Reader, prop_ref: &FileNodeDataRef) -> Result<Self> {
        Ok(Self {
            base: Base::parse(reader, prop_ref)?,
            md5_hash: u128::parse(reader)?,
        })
    }
}

impl<Base: ObjectDeclarationNode> ObjectDeclarationNode
    for ReadOnlyObjectDeclaration2RefCount<Base>
{
    fn id(&self) -> JcId {
        self.base.id()
    }

    fn compact_id(&self) -> CompactId {
        self.base.compact_id()
    }

    fn props(&self) -> Option<&ObjectPropSet> {
        self.base.props()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct UnknownNode {}

impl ParseWithCount for UnknownNode {
    fn parse(reader: Reader, size: usize) -> Result<Self> {
        reader.advance(size)?;
        Ok(UnknownNode {})
    }
}
