use crate::errors::Result;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::desktop::file_node::FileNodeData;
use crate::onestore::desktop::file_node::global_id_table::GlobalIdTableEntry3FNDX;
use crate::onestore::desktop::file_structure::FileNodeDataIterator;
use crate::onestore::desktop::objects::id_mapping::IdMapping;
use crate::onestore::shared::compact_id::CompactId;

/// Lower-level structure for mapping local `CompactId`s to global `ExGuid`s. Applies to a
/// particular region of a OneStore file.
///
/// In `.onetoc2` files, `GlobalIdTable`s may depend on other `GlobalIdTable`s: a
/// [`GlobalIdTableEntry2FNDX`] and [`GlobalIdTableEntry3FNDX`] entries inherit GUIDs from the global
/// identification table of the revision's dependency revision (see [MS-ONESTORE] 2.5.11 and
/// 2.5.12). Such entries are resolved eagerly against `parent` while parsing, so the resulting
/// `id_map` is self-contained.
///
/// See [\[MS-ONESTORE\] 2.1.3](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/a243bd78-6cfd-4e18-96c7-e8c2095ce6b0)
///
/// [`GlobalIdTableEntry2FNDX`]: FileNodeData::GlobalIdTableEntry2FNDX
/// [`GlobalIdTableEntry3FNDX`]: FileNodeData::GlobalIdTableEntry3FNDX
#[derive(Debug, Clone)]
pub(crate) struct GlobalIdTable {
    pub(crate) id_map: IdMapping,
}

impl GlobalIdTable {
    /// Parses a global identification table, resolving any dependency-revision references against
    /// `parent` (the merged `id_map` of the revision this one depends on, if any).
    pub(crate) fn try_parse(
        iterator: &mut FileNodeDataIterator,
        parent: Option<&IdMapping>,
    ) -> Result<Option<Self>> {
        let next = iterator.peek();

        match next {
            Some(
                FileNodeData::GlobalIdTableStart2FND | FileNodeData::GlobalIdTableStartFNDX(_),
            ) => Ok(Some(GlobalIdTable::parse(iterator, parent)?)),
            _ => Ok(None),
        }
    }

    fn parse(iterator: &mut FileNodeDataIterator, parent: Option<&IdMapping>) -> Result<Self> {
        // Skip the start node
        iterator.next();

        let mut id_map = IdMapping::new();

        for node in iterator {
            match node {
                FileNodeData::GlobalIdTableEndFNDX => {
                    break;
                }
                FileNodeData::GlobalIdTableEntryFNDX(entry) => {
                    id_map.add_mapping(entry.index, entry.guid);
                }
                FileNodeData::GlobalIdTableEntry2FNDX(entry) => {
                    // Inherit a GUID from the dependency revision's global identification table.
                    // See [MS-ONESTORE] 2.5.11.
                    match parent.and_then(|p| p.guid_for_index(entry.i_index_map_from)) {
                        Some(guid) => id_map.add_mapping(entry.i_index_map_to, guid),
                        None => {
                            return Err(onestore_parse_error!(
                                "GlobalIdTableEntry2FNDX references index {} that is not present \
                                 in the dependency revision's global ID table",
                                entry.i_index_map_from
                            )
                            .into());
                        }
                    }
                }
                FileNodeData::GlobalIdTableEntry3FNDX(entry) => {
                    copy_parent_range(entry, parent, &mut id_map)?;
                }
                FileNodeData::UnknownNode(_node) => {
                    log::warn!(
                        "Unknown node {:?} skipped while parsing global ID table.",
                        node
                    );
                }
                _ => {
                    return Err(onestore_parse_error!(
                        "Unexpected node ({:?}) encountered while parsing global ID table",
                        node
                    )
                    .into());
                }
            }
        }

        Ok(Self { id_map })
    }

    pub(crate) fn resolve_id(&self, id: &CompactId) -> Result<ExGuid> {
        self.id_map.resolve_id(id)
    }
}

fn copy_parent_range(
    entry: &GlobalIdTableEntry3FNDX,
    parent: Option<&IdMapping>,
    id_map: &mut IdMapping,
) -> Result<()> {
    let Some(parent) = parent else {
        return Err(onestore_parse_error!(
            "GlobalIdTableEntry3FNDX requires a dependency revision's global ID table"
        )
        .into());
    };

    if entry.c_entries_to_copy == 0 {
        return Ok(());
    }

    let last_offset = entry.c_entries_to_copy - 1;
    entry
        .i_index_copy_from_start
        .checked_add(last_offset)
        .ok_or_else(|| {
            onestore_parse_error!(
                "GlobalIdTableEntry3FNDX source range overflows: start {}, count {}",
                entry.i_index_copy_from_start,
                entry.c_entries_to_copy
            )
        })?;
    let destination_end = entry
        .i_index_copy_to_start
        .checked_add(last_offset)
        .filter(|index| *index < 0x00FF_FFFF)
        .ok_or_else(|| {
            onestore_parse_error!(
                "GlobalIdTableEntry3FNDX destination range is outside the 24-bit index space: \
                 start {}, count {}",
                entry.i_index_copy_to_start,
                entry.c_entries_to_copy
            )
        })?;

    debug_assert!(destination_end >= entry.i_index_copy_to_start);

    for offset in 0..entry.c_entries_to_copy {
        let source_index = entry.i_index_copy_from_start + offset;
        let destination_index = entry.i_index_copy_to_start + offset;
        let guid = parent.guid_for_index(source_index).ok_or_else(|| {
            onestore_parse_error!(
                "GlobalIdTableEntry3FNDX references source index {} that is not present in the \
                 dependency revision's global ID table",
                source_index
            )
        })?;
        id_map.add_mapping(destination_index, guid);
    }

    Ok(())
}

impl Default for GlobalIdTable {
    fn default() -> Self {
        Self {
            id_map: IdMapping::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::guid::Guid;
    use uuid::Uuid;

    #[test]
    fn copies_consecutive_parent_mappings() {
        let first = Guid(Uuid::from_u128(1));
        let second = Guid(Uuid::from_u128(2));
        let mut parent = IdMapping::new();
        parent.add_mapping(4, first);
        parent.add_mapping(5, second);
        let mut current = IdMapping::new();

        copy_parent_range(
            &GlobalIdTableEntry3FNDX {
                i_index_copy_from_start: 4,
                c_entries_to_copy: 2,
                i_index_copy_to_start: 10,
            },
            Some(&parent),
            &mut current,
        )
        .unwrap();

        assert_eq!(current.guid_for_index(10), Some(first));
        assert_eq!(current.guid_for_index(11), Some(second));
    }

    #[test]
    fn rejects_missing_source_mapping() {
        let parent = IdMapping::new();
        let mut current = IdMapping::new();

        let error = copy_parent_range(
            &GlobalIdTableEntry3FNDX {
                i_index_copy_from_start: 4,
                c_entries_to_copy: 1,
                i_index_copy_to_start: 10,
            },
            Some(&parent),
            &mut current,
        )
        .unwrap_err();

        assert!(error.to_string().contains("source index 4"));
    }

    #[test]
    fn rejects_destination_outside_compact_id_index_space() {
        let parent = IdMapping::new();
        let mut current = IdMapping::new();

        let error = copy_parent_range(
            &GlobalIdTableEntry3FNDX {
                i_index_copy_from_start: 0,
                c_entries_to_copy: 2,
                i_index_copy_to_start: 0x00FF_FFFE,
            },
            Some(&parent),
            &mut current,
        )
        .unwrap_err();

        assert!(error.to_string().contains("24-bit index space"));
    }
}
