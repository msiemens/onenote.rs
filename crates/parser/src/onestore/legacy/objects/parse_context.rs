use crate::errors::{Error, Result};
use crate::fsshttpb::data::cell_id::CellId;
use crate::fsshttpb::data::exguid::ExGuid;
use crate::onestore::MappingTable;
use crate::onestore::legacy::file_node::shared::AttachmentInfo;
use crate::onestore::legacy::objects::file_data_store::FileDataStore;
use crate::onestore::legacy::objects::global_id_table::GlobalIdTable;
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

    pub(crate) fn find_file_data<'b>(
        &'a self,
        data_info: &'b AttachmentInfo,
    ) -> Result<&'a FileBlob> {
        let file_data_store = self.file_data_store.ok_or_else(|| -> Error {
            parser_error!(ResolutionFailed, "file_data reference has not been loaded").into()
        })?;

        file_data_store.find_file(data_info)
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
