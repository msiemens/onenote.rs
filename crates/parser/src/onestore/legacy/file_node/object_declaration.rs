use crate::Reader;
use crate::onestore::legacy::common::ObjectDeclarationWithRefCountBody;
use crate::onestore::legacy::file_node::FileNodeDataRef;
use crate::onestore::legacy::file_node::shared::{
    AttachmentInfo, ObjectDeclaration2RefCount, ObjectDeclarationNode, ParseWithRef,
    ReadOnlyObjectDeclaration2RefCount, StringInStorageBuffer, read_property_set,
};
use crate::onestore::legacy::parse::Parse;
use crate::onestore::shared::compact_id::CompactId;
use crate::onestore::shared::jcid::JcId;
use crate::onestore::shared::object_prop_set::ObjectPropSet;
use parser_macros::Parse;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ObjectDeclarationWithSizedRefCount<RefSize: Parse> {
    body: ObjectDeclarationWithRefCountBody,
    c_ref: RefSize,
    property_set: ObjectPropSet,
}

impl<RefSize: Parse> ObjectDeclarationNode for ObjectDeclarationWithSizedRefCount<RefSize> {
    fn id(&self) -> JcId {
        self.body.id(true)
    }

    fn compact_id(&self) -> CompactId {
        self.body.oid
    }

    fn props(&self) -> Option<&ObjectPropSet> {
        Some(&self.property_set)
    }
}

impl<'a, RefSize: Parse> ParseWithRef<'a> for ObjectDeclarationWithSizedRefCount<RefSize> {
    fn parse(reader: Reader, data_ref: &FileNodeDataRef) -> crate::errors::Result<Self> {
        let property_set = read_property_set(reader, data_ref)?;

        Ok(Self {
            body: ObjectDeclarationWithRefCountBody::parse(reader)?,
            c_ref: RefSize::parse(reader)?,
            property_set,
        })
    }
}
pub(crate) type ObjectDeclarationWithRefCountFNDX = ObjectDeclarationWithSizedRefCount<u8>;

pub(crate) type ObjectDeclarationWithRefCount2FNDX = ObjectDeclarationWithSizedRefCount<u32>;

/// See [\[MS-ONESTORE\] 2.5.27](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/da2bbc7d-0529-4bf4-a843-6f3f55c87e8f)
#[derive(Debug, Clone, Parse)]
#[validate({
    let data = &file_data_ref.data;
    data.starts_with("<file>") || data.starts_with("<ifndf>") || data.starts_with("<invfdo>")
})]
#[allow(dead_code)]
pub(crate) struct ObjectDeclarationFileDataRefCount<RefSize: Parse> {
    oid: CompactId,
    jcid: JcId,
    #[assert_offset(8)]
    c_ref: RefSize,
    file_data_ref: StringInStorageBuffer,
    file_ext: StringInStorageBuffer,
}

impl<RefSize: Parse> ObjectDeclarationNode for ObjectDeclarationFileDataRefCount<RefSize> {
    fn id(&self) -> JcId {
        self.jcid
    }

    fn compact_id(&self) -> CompactId {
        self.oid
    }

    fn props(&self) -> Option<&ObjectPropSet> {
        None
    }

    fn get_attachment_info(&self) -> Option<AttachmentInfo> {
        Some(AttachmentInfo {
            data_ref: self.file_data_ref.data.clone(),
            extension: self.file_ext.data.clone(),
        })
    }
}

pub(crate) type ObjectDeclarationFileData3RefCountFND = ObjectDeclarationFileDataRefCount<u8>;
pub(crate) type ObjectDeclarationFileData3LargeRefCountFND = ObjectDeclarationFileDataRefCount<u32>;

pub(crate) type ObjectDeclaration2RefCountFND = ObjectDeclaration2RefCount<u8>;

pub(crate) type ObjectDeclaration2LargeRefCountFND = ObjectDeclaration2RefCount<u32>;

pub(crate) type ReadOnlyObjectDeclaration2RefCountFND =
    ReadOnlyObjectDeclaration2RefCount<ObjectDeclaration2RefCountFND>;

pub(crate) type ReadOnlyObjectDeclaration2LargeRefCountFND =
    ReadOnlyObjectDeclaration2RefCount<ObjectDeclaration2LargeRefCountFND>;
