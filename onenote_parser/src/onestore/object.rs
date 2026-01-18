use super::mapping_table::MappingTable;
use crate::{
    one::property_set::PropertySetId,
    shared::{
        exguid::ExGuid, file_data_ref::FileBlob, jcid::JcId, object_prop_set::ObjectPropSet,
    },
};
use crate::utils::Result;
use std::rc::Rc;

pub trait ObjectFileData {
    fn load(&self) -> Result<FileBlob>;
}

impl ObjectFileData for FileBlob {
    fn load(&self) -> Result<FileBlob> {
        Ok(self.clone())
    }
}

/// See [\[MS-ONESTORE\] 2.1.5](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/ce60b62f-82e5-401a-bf2c-3255457732ad)
#[derive(Clone)]
pub(crate) struct Object {
    pub(crate) context_id: ExGuid,

    pub(crate) jc_id: JcId,
    pub(crate) props: ObjectPropSet,
    pub(crate) file_data: Option<Rc<dyn ObjectFileData>>,
    pub(crate) mapping: Rc<dyn MappingTable>,
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Object");
        if let Some(id) = &PropertySetId::from_jcid(self.jc_id) {
            debug.field("id", id);
        } else {
            // Unknown object type
            debug.field("id", &self.jc_id);
        }

        if let Some(info) = &self.file_data {
            debug.field("file_data", &info.load());
        }

        debug.field("props", self.props.properties());
        debug.finish()
    }
}

impl Object {
    pub fn id(&self) -> JcId {
        self.jc_id
    }

    pub fn props(&self) -> &ObjectPropSet {
        &self.props
    }
}
