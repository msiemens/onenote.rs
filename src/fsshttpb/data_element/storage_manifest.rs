use crate::errors::Result;
use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::fsshttpb::data::object_types::ObjectType;
use crate::fsshttpb::data::stream_object::ObjectHeader;
use crate::fsshttpb::data_element::DataElement;
use crate::shared::guid::Guid;
use crate::Reader;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct StorageManifest {
    pub(crate) id: Guid,
    pub(crate) roots: HashMap<ExGuid, CellId>,
}

impl DataElement {
    pub(crate) fn parse_storage_manifest(reader: Reader) -> Result<StorageManifest> {
        ObjectHeader::try_parse_16(reader, ObjectType::StorageManifest)?;

        let id = Guid::parse(reader)?;

        let mut roots = HashMap::new();

        loop {
            if ObjectHeader::has_end_8(reader, ObjectType::DataElement)? {
                break;
            }

            ObjectHeader::try_parse_16(reader, ObjectType::StorageManifestRoot)?;

            let root_manifest = ExGuid::parse(reader)?;
            let cell = CellId::parse(reader)?;

            roots.insert(root_manifest, cell);
        }

        ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement)?;

        Ok(StorageManifest { id, roots })
    }
}
