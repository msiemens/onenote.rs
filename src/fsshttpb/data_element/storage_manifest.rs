use crate::fsshttpb::data_element::DataElement;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;
use crate::types::object_types::ObjectType;
use crate::types::stream_object::ObjectHeader;
use crate::Reader;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct StorageManifest {
    pub(crate) id: Guid,
    pub(crate) roots: HashMap<ExGuid, CellId>,
}

impl DataElement {
    pub(crate) fn parse_storage_manifest(reader: Reader) -> StorageManifest {
        let object_header = ObjectHeader::parse_16(reader);
        assert_eq!(object_header.object_type, ObjectType::StorageManifest);

        let id = Guid::parse(reader);

        let mut roots = HashMap::new();

        loop {
            if ObjectHeader::try_parse_end_8(reader, ObjectType::DataElement).is_some() {
                break;
            }

            let object_header = ObjectHeader::parse_16(reader);
            assert_eq!(object_header.object_type, ObjectType::StorageManifestRoot);

            let root_manifest = ExGuid::parse(reader);
            let cell = CellId::parse(reader);

            roots.insert(root_manifest, cell);
        }

        StorageManifest { id, roots }
    }
}
