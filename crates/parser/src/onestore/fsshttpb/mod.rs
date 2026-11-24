use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::data_element::storage_manifest::StorageManifest;
use crate::fsshttpb::packaging::OneStorePackaging;
use crate::onestore::fsshttpb::header::StoreHeader;
use crate::onestore::fsshttpb::object_space::ObjectSpace;
use crate::onestore::fsshttpb::revision::Revision;
use crate::onestore::{OneStore, OneStoreType};
use crate::shared::guid::Guid;
use std::collections::{HashMap, HashSet, VecDeque};

pub(crate) mod header;
pub(crate) mod mapping_table;
pub(crate) mod object;
pub(crate) mod object_space;
pub(crate) mod revision;
mod revision_role;

/// A OneNote file packaged in a FSSHTTPB envelope.
///
/// See [\[MS-ONESTORE\] 2.7]
///
/// [\[MS-ONESTORE\] 2.7]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/189f186c-84ea-4892-afca-633c22bf9389
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct PackagingStore {
    schema: Guid,
    header: StoreHeader,
    data_root: ObjectSpace,
    object_spaces: HashMap<CellId, ObjectSpace>,
}

impl PackagingStore {
    pub(crate) fn data_root(&self) -> &ObjectSpace {
        &self.data_root
    }

    pub(crate) fn is_onestore(&self) -> bool {
        self.schema == guid!("{1F937CB4-B26F-445F-B9F8-17E20160E461}")
    }
}

impl OneStore for PackagingStore {
    fn file_identity(&self) -> Guid {
        self.header.file_identity()
    }

    fn get_type(&self) -> Result<OneStoreType> {
        if self.is_onestore() {
            Ok(OneStoreType::Section)
        } else {
            Ok(OneStoreType::TableOfContents)
        }
    }

    fn data_root(&self) -> &dyn crate::onestore::ObjectSpace {
        &self.data_root
    }

    fn object_space(&self, id: CellId) -> Option<&dyn crate::onestore::ObjectSpace> {
        self.object_spaces
            .get(&id)
            .map(|space| space as &dyn crate::onestore::ObjectSpace)
    }
}

pub(crate) fn parse_store(package: &OneStorePackaging) -> Result<PackagingStore> {
    let mut parsed_object_spaces = HashSet::new();

    // [ONESTORE] 2.7.1: Parse storage manifest
    let storage_index = package
        .data_element_package
        .find_storage_index_by_id(package.storage_index)
        .or_else(|| package.data_element_package.find_storage_index())
        .ok_or_else(|| ErrorKind::MalformedOneStoreData("storage index is missing".into()))?;
    let storage_manifest = package
        .data_element_package
        .find_storage_manifest()
        .ok_or_else(|| ErrorKind::MalformedOneStoreData("storage manifest is missing".into()))?;

    let header_cell_id = find_header_cell_id(storage_manifest)?;

    let header_cell_mapping_id = storage_index
        .find_cell_mapping_id(header_cell_id)
        .ok_or_else(|| {
            ErrorKind::MalformedOneStoreData("header cell mapping id not found".into())
        })?;

    // [ONESTORE] 2.7.2: Parse header cell
    let header_cell = package
        .data_element_package
        .find_objects(header_cell_mapping_id, storage_index)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            ErrorKind::MalformedOneStoreData("no header object in header cell".into())
        })?;

    let header = StoreHeader::parse(header_cell)?;

    parsed_object_spaces.insert(header_cell_id);

    // Revision cache deduplicates already parsed revisions by cell id.
    // Object spaces can reference revisions across cells; caching avoids re-parsing and
    // preserves identity when multiple spaces share the same revision.
    let mut revision_cache = HashMap::new();

    // Parse data root

    let data_root_cell_id = find_data_root_cell_id(storage_manifest)?;
    let (_, data_root) = parse_object_space(
        data_root_cell_id,
        storage_index,
        package,
        &mut revision_cache,
    )?;

    parsed_object_spaces.insert(data_root_cell_id);

    // Parse the object spaces reachable from the data root
    //
    // The storage index can map many more cells than the current revision uses (e.g. old page
    // versions in a section with a long edit history). Only the object spaces that are referenced,
    // directly or transitively, by the data root are ever looked up, so follow those references
    // instead of parsing every cell.

    let mut object_spaces = HashMap::new();
    let mut pending: VecDeque<CellId> = data_root.referenced_object_spaces().into();

    while let Some(cell_id) = pending.pop_front() {
        if !parsed_object_spaces.insert(cell_id) {
            continue;
        }

        // A reference to a missing cell is reported where it is resolved
        let Some(mapping) = storage_index.cell_mappings.get(&cell_id) else {
            continue;
        };

        if mapping.id.is_nil() {
            continue;
        }

        let (id, group) =
            parse_object_space(mapping.cell_id, storage_index, package, &mut revision_cache)?;
        pending.extend(group.referenced_object_spaces());
        object_spaces.insert(id, group);
    }

    Ok(PackagingStore {
        schema: storage_manifest.id,
        header,
        data_root,
        object_spaces,
    })
}

fn parse_object_space<'a>(
    cell_id: CellId,
    storage_index: &'a StorageIndex,
    package: &'a OneStorePackaging,
    revision_cache: &mut HashMap<CellId, Revision>,
) -> Result<(CellId, ObjectSpace)> {
    let mapping = storage_index
        .cell_mappings
        .get(&cell_id)
        .ok_or_else(|| ErrorKind::MalformedOneStoreData("cell mapping not found".into()))?;

    ObjectSpace::parse(mapping, storage_index, package, revision_cache)
}

fn find_header_cell_id(manifest: &StorageManifest) -> Result<CellId> {
    manifest
        .roots
        .get(&exguid!({{"1A5A319C-C26B-41AA-B9C5-9BD8C44E07D4"}, 1}))
        .copied()
        .ok_or_else(|| ErrorKind::MalformedOneStoreData("no header cell root".into()).into())
}

fn find_data_root_cell_id(manifest: &StorageManifest) -> Result<CellId> {
    manifest
        .roots
        .get(&exguid!({{"84DEFAB9-AAA3-4A0D-A3A8-520C77AC7073"}, 2}))
        .copied()
        .ok_or_else(|| ErrorKind::MalformedOneStoreData("no header cell root".into()).into())
}

#[cfg(test)]
mod tests {
    use super::parse_store;
    use crate::fsshttpb::data::cell_id::CellId;
    use crate::fsshttpb::packaging::OneStorePackaging;
    use crate::reader::Reader;
    use std::collections::HashSet;

    #[test]
    fn parses_only_object_spaces_reachable_from_the_data_root() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/samples/joplin/aaa/Quick Notes.one"
        );
        let data = std::fs::read(path).unwrap();
        let packaging = OneStorePackaging::parse(&mut Reader::new(&data)).unwrap();
        let store = parse_store(&packaging).unwrap();

        // Follow the object space references from the data root through the parsed spaces
        let mut reachable = HashSet::new();
        let mut pending = store.data_root.referenced_object_spaces();
        while let Some(cell_id) = pending.pop() {
            if !reachable.insert(cell_id) {
                continue;
            }
            if let Some(space) = store.object_spaces.get(&cell_id) {
                pending.extend(space.referenced_object_spaces());
            }
        }

        let storage_index = packaging.data_element_package.find_storage_index().unwrap();
        let mapped: HashSet<CellId> = storage_index
            .cell_mappings
            .values()
            .filter(|mapping| !mapping.id.is_nil())
            .map(|mapping| mapping.cell_id)
            .collect();
        let parsed: HashSet<CellId> = store.object_spaces.keys().copied().collect();

        // Every reachable space is parsed, and nothing else
        assert_eq!(parsed, &reachable & &mapped);

        // Besides the header and the data root, the sample maps cells that are not reachable
        assert!(mapped.len() - parsed.len() > 2);
    }
}
