use crate::errors::{Error, Result};
use crate::onestore::legacy::file_node::FileNodeData;
use crate::onestore::legacy::file_node::shared::AttachmentInfo;
use crate::onestore::legacy::file_node::shared::FileData;
use crate::onestore::legacy::file_node::shared::FileDataStoreListReferenceFND;
use crate::onestore::legacy::file_structure::FileNodeDataIterator;
use crate::onestore::shared::file_blob::FileBlob;
use crate::shared::guid::Guid;

/// See [MS-ONESTORE 2.5.21](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/2701cc42-3601-49f9-a3ba-7c40cd8a2be9)
#[derive(Debug, Clone)]
pub(crate) struct FileDataStore {
    pub(crate) files: Vec<File>,
}

#[derive(Debug, Clone)]
pub(crate) struct File {
    id: Guid,
    pub(crate) file_data: FileData,
}

impl FileDataStore {
    pub(crate) fn try_parse(iterator: &mut FileNodeDataIterator) -> Result<Option<Self>> {
        if let Some(FileNodeData::FileDataStoreListReferenceFND(data)) = iterator.peek() {
            iterator.next();
            Ok(Some(Self::from_reference(data)?))
        } else {
            Ok(None)
        }
    }

    fn from_reference(reference: &FileDataStoreListReferenceFND) -> Result<Self> {
        let iterator = reference.list.iter_data();
        let mut files = Vec::new();
        for item in iterator {
            if let FileNodeData::FileDataStoreObjectReferenceFND(item) = item {
                files.push(File {
                    id: item.guid,
                    file_data: item.target.file_data.clone(),
                })
            } else {
                return Err(onestore_parse_error!(
                    "Unexpected item in file list: {:?}. Expected FileDataStoreObjectReferenceFND",
                    item
                )
                .into());
            }
        }

        Ok(Self { files })
    }

    pub(crate) fn find_file<'b>(&'b self, info: &AttachmentInfo) -> Result<&'b FileBlob> {
        info.load_data(|id| -> Result<&'b FileBlob> {
            let guid = Guid::from_str(id)?;
            let file = self
                .files
                .iter()
                .find(|file| file.id == guid)
                .ok_or_else(|| -> Error {
                    parser_error!(ResolutionFailed, "File not found with ID {}", id).into()
                })?;

            Ok(&file.file_data.0)
        })
    }
}
