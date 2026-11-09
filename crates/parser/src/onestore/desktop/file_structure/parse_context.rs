use std::collections::HashMap;

use crate::onestore::desktop::file_structure::{
    TransactionLogFragment, file_node_list_fragment::FileNodeListHeader,
};

#[derive(Default)]
pub(crate) struct ParseContext {
    file_node_count_mapping: HashMap<u32, usize>,
}

impl ParseContext {
    pub(crate) fn new() -> Self {
        Self {
            file_node_count_mapping: HashMap::new(),
        }
    }

    /// Returns the maximum number of nodes in the provided node list
    pub(crate) fn get_file_node_count(&self, header: &FileNodeListHeader) -> Option<usize> {
        self.file_node_count_mapping
            .get(&header.file_node_list_id)
            .cloned()
    }

    pub(crate) fn update_remaining_nodes_in_fragment(
        &mut self,
        header: &FileNodeListHeader,
        remaining: usize,
    ) {
        self.file_node_count_mapping
            .insert(header.file_node_list_id, remaining);
    }

    pub(crate) fn update_from_transaction_log(
        &mut self,
        log: &Vec<TransactionLogFragment>,
        committed: u32,
    ) {
        let mut completed = 0u32;
        for fragment in log {
            // See [MS-ONESTORE 2.3.3.2](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/c00897d9-d90a-4707-b9fb-58c93e490322)
            // The transaction entries specify the number of nodes in the FileNodeListFragments.
            for entry in &fragment.size_table {
                if completed >= committed {
                    return;
                }
                if entry.is_sentinel() {
                    completed += 1;
                    continue;
                }

                let new_count = entry.transaction_entry_switch as usize;
                if let Some(current_count) = self.file_node_count_mapping.get(&entry.src_id) {
                    if *current_count < new_count {
                        self.file_node_count_mapping.insert(entry.src_id, new_count);
                    }
                } else {
                    self.file_node_count_mapping.insert(entry.src_id, new_count);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::onestore::desktop::file_structure::transaction_log_fragment::TransactionEntry;

    fn fragment(entries: &[(u32, u32)]) -> TransactionLogFragment {
        TransactionLogFragment {
            size_table: entries
                .iter()
                .map(|&(src_id, transaction_entry_switch)| TransactionEntry {
                    src_id,
                    transaction_entry_switch,
                })
                .collect(),
            next_fragment: crate::onestore::desktop::common::FileChunkReference64x32 {
                stp: u64::MAX,
                cb: 0,
            },
        }
    }

    #[test]
    fn counts_after_the_last_committed_transaction_are_stale_and_read_as_nothing() {
        // One committed transaction (the sentinel, source id 1), then a stale tail.
        let log = vec![fragment(&[(0x97, 14), (1, 0), (0x97, 9000), (0x98, 9000)])];
        let mut context = ParseContext::new();
        context.update_from_transaction_log(&log, 1);
        assert_eq!(context.file_node_count_mapping.get(&0x97), Some(&14));
        assert_eq!(context.file_node_count_mapping.get(&0x98), None);
    }
}
