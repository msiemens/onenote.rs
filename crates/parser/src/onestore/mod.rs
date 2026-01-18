use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::one::property_set::PropertySetId;
use shared::compact_id::CompactId;
use shared::file_blob::FileBlob;
use shared::jcid::JcId;
use shared::object_prop_set::ObjectPropSet;
use std::fmt;
use std::rc::Rc;

pub mod fsshttpb;
pub mod legacy;
pub mod shared;

pub(crate) trait OneStore {
    fn get_type(&self) -> OneStoreType;

    fn data_root(&self) -> &dyn ObjectSpace;

    /// Fetch the object space that is parent to the object identified by the
    /// given `id` (if any).
    fn object_space(&self, id: CellId) -> Option<&dyn ObjectSpace>;
}

#[derive(Eq, PartialEq)]
pub(crate) enum OneStoreType {
    TableOfContents, // .onetoc2
    Section,         // .one
}

pub(crate) trait ObjectSpace: fmt::Debug {
    fn get_object(&self, id: ExGuid) -> Option<&Object>;

    fn content_root(&self) -> Option<ExGuid>;

    fn metadata_root(&self) -> Option<ExGuid>;
}

pub(crate) trait MappingTable {
    fn resolve_id(&self, index: usize, cid: &CompactId) -> Option<ExGuid>;

    fn get_object_space(&self, index: usize, cid: &CompactId) -> Option<CellId>;
}

impl fmt::Debug for dyn MappingTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[MappingTable]")
    }
}

/// A OneNote data object.
///
/// See [\[MS-ONESTOR\] 2.1.5] and [\[MS-ONESTOR\] 2.7.6]
///
/// [\[MS-ONESTOR\] 2.1.5]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/ce60b62f-82e5-401a-bf2c-3255457732ad
/// [\[MS-ONESTOR\] 2.7.6]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/b4270940-827e-468b-bf42-2c7afee23740
#[derive(Clone)]
pub(crate) struct Object {
    pub(crate) context_id: ExGuid,

    pub(crate) jc_id: JcId,
    pub(crate) props: ObjectPropSet,
    pub(crate) file_data: Option<FileBlob>,
    pub(crate) mapping: Rc<dyn MappingTable>,
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Object");
        if let Some(id) = &PropertySetId::from_jcid(self.jc_id) {
            debug.field("id", id);
        } else {
            // Unknown object type
            debug.field("id", &self.jc_id);
        }

        if let Some(info) = &self.file_data {
            debug.field("file_data", &info.as_ref());
        }

        debug.field("props", self.props.properties());
        debug.finish()
    }
}

impl Object {
    pub fn id(&self) -> JcId {
        self.jc_id
    }
}
