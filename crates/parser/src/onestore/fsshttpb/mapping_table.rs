use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::shared::compact_id::CompactId;
use std::collections::HashMap;

/// The ID mapping table for an object.
///
/// The specification isn't really clear on how the mapping table works. According to the spec,
/// the mapping table maps from `CompactId`s to `ExGuid`s for objects and `CellId`s for object
/// spaces. BUT while it specifies how to build the mapping table, it doesn't mention how it
/// is used. From testing it looks like there cases where a single `CompactId` maps to *multiple*
/// `ExGuid`s/`CellId`s. In this case we will use the table _index_ as a fallback.
///
/// See [\[MS-ONESTORE\] 2.7.8].
///
/// [\[MS-ONESTORE\] 2.7.8]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/c2e58ac6-7a86-4009-a1e4-4a84cd21508f
#[derive(Debug, Clone)]
pub(crate) struct MappingTable {
    objects: HashMap<CompactId, Vec<(usize, ExGuid)>>,
    object_spaces: HashMap<CompactId, Vec<(usize, CellId)>>,
}

impl crate::onestore::MappingTable for MappingTable {
    fn resolve_id(&self, index: usize, cid: &CompactId) -> Option<ExGuid> {
        self.get(index, *cid, &self.objects)
    }

    fn get_object_space(&self, index: usize, cid: &CompactId) -> Option<CellId> {
        self.get(index, *cid, &self.object_spaces)
    }
}

impl MappingTable {
    pub(crate) fn from_entries<
        I: Iterator<Item = (CompactId, ExGuid)>,
        J: Iterator<Item = (CompactId, CellId)>,
    >(
        objects: I,
        object_spaces: J,
    ) -> MappingTable {
        let mut objects_map: HashMap<CompactId, Vec<(usize, ExGuid)>> = HashMap::new();
        for (i, (cid, id)) in objects.enumerate() {
            objects_map.entry(cid).or_default().push((i, id));
        }

        let mut object_spaces_map: HashMap<CompactId, Vec<(usize, CellId)>> = HashMap::new();
        for (i, (cid, id)) in object_spaces.enumerate() {
            object_spaces_map.entry(cid).or_default().push((i, id));
        }

        MappingTable {
            objects: objects_map,
            object_spaces: object_spaces_map,
        }
    }

    fn get<T: Copy>(
        &self,
        index: usize,
        cid: CompactId,
        table: &HashMap<CompactId, Vec<(usize, T)>>,
    ) -> Option<T> {
        if let Some(entries) = table.get(&cid) {
            // Only one entry: return it!
            if let [(_, id)] = &**entries {
                return Some(*id);
            }

            // Find entry with matching table index
            if let Some((_, id)) = entries.iter().find(|(i, _)| *i == index) {
                return Some(*id);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::MappingTable;
    use crate::fsshttpb::data::cell_id::CellId;
    use crate::fsshttpb::data::exguid::ExGuid;
    use crate::onestore::MappingTable as _;
    use crate::onestore::shared::compact_id::CompactId;
    use crate::reader::Reader;
    use crate::shared::guid::Guid;

    fn compact_id(n: u8, guid_index: u32) -> CompactId {
        let data = (guid_index << 8) | n as u32;
        CompactId::parse(&mut Reader::new(&data.to_le_bytes())).unwrap()
    }

    fn exguid(value: u32) -> ExGuid {
        ExGuid::from_guid(Guid::nil(), value)
    }

    #[test]
    fn test_single_entry_fallbacks_to_only_value() {
        let cid = compact_id(1, 0x10);
        let object = exguid(10);
        let table = MappingTable::from_entries(
            vec![(cid, object)].into_iter(),
            std::iter::empty::<(CompactId, CellId)>(),
        );

        assert_eq!(table.resolve_id(0, &cid), Some(object));
        assert_eq!(table.resolve_id(3, &cid), Some(object));
    }

    #[test]
    fn test_multiple_entries_match_by_index() {
        let cid = compact_id(2, 0x20);
        let first = exguid(1);
        let second = exguid(2);
        let table = MappingTable::from_entries(
            vec![(cid, first), (cid, second)].into_iter(),
            std::iter::empty::<(CompactId, CellId)>(),
        );

        assert_eq!(table.resolve_id(0, &cid), Some(first));
        assert_eq!(table.resolve_id(1, &cid), Some(second));
        assert_eq!(table.resolve_id(2, &cid), None);
    }

    #[test]
    fn test_object_space_lookup() {
        let cid = compact_id(3, 0x30);
        let cell_a = CellId(exguid(10), exguid(11));
        let cell_b = CellId(exguid(20), exguid(21));
        let table = MappingTable::from_entries(
            std::iter::empty::<(CompactId, ExGuid)>(),
            vec![(cid, cell_a), (cid, cell_b)].into_iter(),
        );

        assert_eq!(table.get_object_space(0, &cid), Some(cell_a));
        assert_eq!(table.get_object_space(1, &cid), Some(cell_b));
        assert_eq!(table.get_object_space(2, &cid), None);
    }
}
