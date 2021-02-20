use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::types::compact_id::CompactId;
use std::collections::HashMap;

/// The ID mapping table for an object.
///
/// The specification isn't really clear on how the mapping table works. According to the spec,
/// the mapping table maps from `CompactId`s to `ExGuid`s for objects and `CellId`s for object
/// spaces. BUT while it specifies how to build the mapping table, it doesn't mention how it
/// is used. From testing it looks like one should use the table _index_ to look up IDs, not the
/// `CompactId`. This would be the way how the global ID table works with regular OneStore files
/// which map an offset to an ID.
///
/// See [\[MS-ONESTORE 2.7.8\]].
///
/// [\[MS-ONESTORE 2.7.8\]]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/c2e58ac6-7a86-4009-a1e4-4a84cd21508f
#[derive(Debug, Clone)]
pub(crate) struct MappingTable {
    objects: HashMap<usize, (CompactId, ExGuid)>,
    object_spaces: HashMap<usize, (CompactId, CellId)>,
}

impl MappingTable {
    pub(crate) fn from_entries<
        I: Iterator<Item = (CompactId, ExGuid)>,
        J: Iterator<Item = (CompactId, CellId)>,
    >(
        objects: I,
        object_spaces: J,
    ) -> MappingTable {
        MappingTable {
            objects: objects.enumerate().collect(),
            object_spaces: object_spaces.enumerate().collect(),
        }
    }

    pub(crate) fn get_object(&self, index: usize, id: CompactId) -> Option<ExGuid> {
        self.objects
            .get(&index)
            .copied()
            .map(|(entry_id, target_id)| {
                assert_eq!(entry_id, id);

                target_id
            })
    }

    pub(crate) fn get_object_space(&self, index: usize, id: CompactId) -> Option<CellId> {
        self.object_spaces
            .get(&index)
            .copied()
            .map(|(entry_id, target_id)| {
                assert_eq!(entry_id, id);

                target_id
            })
    }
}
