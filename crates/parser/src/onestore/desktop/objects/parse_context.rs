use crate::errors::{Error, Result};
use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::MappingTable;
use crate::onestore::desktop::file_node::shared::AttachmentInfo;
use crate::onestore::desktop::objects::file_data_store::FileDataStore;
use crate::onestore::desktop::objects::global_id_table::GlobalIdTable;
use crate::onestore::shared::compact_id::CompactId;
use crate::onestore::shared::file_blob::FileBlob;
use std::rc::Rc;

/// Provides an interface to access shared data to mid-level parsing logic.
/// Use this, for example, to resolve IDs.
#[derive(Clone)]
pub(crate) struct ParseContext<'a> {
    pub(crate) id_map: Rc<dyn MappingTable>,

    /// The ID of the ObjectSpace that contains the current node
    pub(crate) context_id: ExGuid,

    file_data_store: Option<&'a FileDataStore>,
}

impl<'a> ParseContext<'a> {
    pub(crate) fn new() -> Self {
        Self {
            id_map: Rc::new(ParseContextIdMapping::default()),
            file_data_store: None,
            context_id: exguid!({{"00000000-0000-0000-0000-000000000000"}, 0}),
        }
    }

    pub(crate) fn with_id_table(&self, id_table: &GlobalIdTable) -> Self {
        Self {
            id_map: Rc::new(ParseContextIdMapping::new(id_table)),
            file_data_store: self.file_data_store,
            context_id: self.context_id,
        }
    }

    pub(crate) fn with_context_id(&self, context_id: ExGuid) -> Self {
        Self {
            id_map: self.id_map.clone(),
            file_data_store: self.file_data_store,
            context_id,
        }
    }

    pub(crate) fn with_file_data_store<'b>(
        &self,
        file_data_store: &'b FileDataStore,
    ) -> ParseContext<'b> {
        ParseContext {
            id_map: self.id_map.clone(),
            file_data_store: Some(file_data_store),
            context_id: self.context_id,
        }
    }

    pub(crate) fn find_file_data(&self, data_info: &AttachmentInfo) -> Result<FileBlob> {
        data_info.load_data(|id| {
            let file_data_store = self.file_data_store.ok_or_else(|| -> Error {
                parser_error!(ResolutionFailed, "file_data reference has not been loaded").into()
            })?;
            file_data_store.find_file(id)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ParseContext;
    use crate::onestore::desktop::file_node::shared::AttachmentInfo;
    use crate::onestore::shared::file_blob::FileDataStatus;

    #[test]
    fn invalid_file_reference_does_not_require_a_file_data_store() {
        let info = AttachmentInfo {
            extension: "bin".to_owned(),
            data_ref: "<invfdo>".to_owned(),
        };

        let blob = ParseContext::new()
            .find_file_data(&info)
            .expect("invalid file references should remain non-fatal");

        assert_eq!(blob.status(), FileDataStatus::Invalid);
        assert_eq!(blob.size(), 0);
    }

    #[test]
    fn internal_file_reference_still_requires_a_file_data_store() {
        let info = AttachmentInfo {
            extension: "bin".to_owned(),
            data_ref: "<ifndf>{00000000-0000-0000-0000-000000000000}".to_owned(),
        };

        let error = ParseContext::new()
            .find_file_data(&info)
            .expect_err("internal references require the file data store");

        assert!(
            error
                .to_string()
                .contains("file_data reference has not been loaded")
        );
    }
}

#[derive(Clone, Default)]
struct ParseContextIdMapping {
    id_table: GlobalIdTable,
}

impl ParseContextIdMapping {
    pub(crate) fn new(id_table: &GlobalIdTable) -> Self {
        Self {
            id_table: id_table.clone(),
        }
    }
}

impl MappingTable for ParseContextIdMapping {
    fn resolve_id(&self, _index: usize, cid: &CompactId) -> Option<ExGuid> {
        self.id_table.resolve_id(cid).ok()
    }

    fn get_object_space(&self, _index: usize, cid: &CompactId) -> Option<CellId> {
        if let Ok(result) = self.id_table.resolve_id(cid) {
            Some(CellId(result, ExGuid::default()))
        } else {
            None
        }
    }
}
