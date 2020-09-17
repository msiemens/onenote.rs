use crate::errors::Result;
use crate::fsshttpb::data_element::storage_index::StorageIndex;
use crate::fsshttpb::data_element::storage_manifest::StorageManifest;
use crate::fsshttpb::data_element::value::DataElementValue;
use crate::fsshttpb::packaging::Packaging;
use crate::onestore::header::StoreHeader;
use crate::onestore::object_space::ObjectSpace;
use crate::types::cell_id::CellId;
use crate::types::exguid::ExGuid;
use crate::types::guid::Guid;
use std::collections::{HashMap, HashSet};

mod header;
mod mapping_table;
pub(crate) mod object;
pub(crate) mod object_group;
pub(crate) mod object_space;
pub(crate) mod revision;
mod types;

#[derive(Debug)]
pub(crate) struct OneStore {
    header: StoreHeader,
    data_root: ObjectSpace,
    object_spaces: HashMap<ExGuid, ObjectSpace>,
}

pub(crate) fn parse_store(package: Packaging) -> Result<OneStore> {
    let mut parsed_object_spaces = HashSet::new();

    // [ONESTORE] 2.7.1: Parse storage manifest
    let storage_index = find_storage_index(&package);
    let storage_manifest = find_storage_manifest(&package);

    let header_cell_id = find_header_cell_id(storage_manifest);

    let header_cell_mapping_id = storage_index
        .find_cell_mapping_id(header_cell_id)
        .expect("header cell mapping id not found");

    // [ONESTORE] 2.7.2: Parse header cell
    let header_cell = package
        .data_element_package
        .find_objects(header_cell_mapping_id, &storage_index)
        .into_iter()
        .next()
        .expect("no header object in header cell");

    let header = StoreHeader::parse(header_cell);

    parsed_object_spaces.insert(header_cell_id);

    // Parse data root

    let data_root_cell_id = find_data_root_cell_id(storage_manifest);
    let (_, data_root) = parse_object_space(data_root_cell_id, storage_index, &package);

    parsed_object_spaces.insert(data_root_cell_id);

    // Parse other object spaces

    let mut object_spaces = HashMap::new();

    for mapping in &storage_index.cell_mappings {
        if parsed_object_spaces.contains(&mapping.cell_id) {
            continue;
        }

        let (id, group) = parse_object_space(mapping.cell_id, storage_index, &package);
        object_spaces.insert(id, group);
    }

    Ok(OneStore {
        header,
        data_root,
        object_spaces,
    })
}

fn parse_object_space(
    cell_id: CellId,
    storage_index: &StorageIndex,
    package: &Packaging,
) -> (ExGuid, ObjectSpace) {
    let mapping = storage_index
        .cell_mappings
        .iter()
        .find(|mapping| mapping.cell_id == cell_id)
        .expect("cell mapping not found");

    ObjectSpace::parse(mapping, storage_index, package)
}

fn find_storage_index(package: &Packaging) -> &StorageIndex {
    package
        .data_element_package
        .elements
        .iter()
        .find_map(|element| {
            if let DataElementValue::StorageIndex(index) = &element.element {
                Some(index)
            } else {
                None
            }
        })
        .expect("no storage index found")
}

fn find_storage_manifest(package: &Packaging) -> &StorageManifest {
    package
        .data_element_package
        .elements
        .iter()
        .find_map(|element| {
            if let DataElementValue::StorageManifest(manifest) = &element.element {
                Some(manifest)
            } else {
                None
            }
        })
        .expect("no storage manifest found")
}

fn find_header_cell_id(manifest: &StorageManifest) -> CellId {
    manifest
        .roots
        .iter()
        .find(|root| {
            root.root_manifest
                == ExGuid::from_guid(
                    Guid::from_str("1A5A319C-C26b-41AA-B9C5-9BD8C44E07D4").unwrap(),
                    1,
                )
        })
        .expect("no header cell root")
        .cell
}

fn find_data_root_cell_id(manifest: &StorageManifest) -> CellId {
    manifest
        .roots
        .iter()
        .find(|root| {
            root.root_manifest
                == ExGuid::from_guid(
                    Guid::from_str("84DEFAB9-AAA3-4A0D-A3A8-520C77AC7073").unwrap(),
                    2,
                )
        })
        .expect("no header cell root")
        .cell
}
