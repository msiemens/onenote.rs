use super::super::common::{FileChunkReference64, FileChunkReference64x32};
use crate::onestore::legacy::parse::Parse;
use crate::{Reader, errors::Result};

/// See [\[MS-ONESTORE\] 2.3.2.1](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/904a92ff-5c38-48d5-b01e-768846e38083)
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct FreeChunkListFragment {
    size: u64,
    crc: u32,
    pub(crate) fcr_next_chunk: FileChunkReference64x32,
    fcr_free_chunk: Vec<FileChunkReference64>,
}

impl FreeChunkListFragment {
    pub(crate) fn parse(reader: Reader, size: u64) -> Result<Self> {
        let crc = reader.get_u32()?;
        let fcr_next_chunk = FileChunkReference64x32::parse(reader)?;

        // Length of the free_chunk (see \[MS-ONESTORE\] 2.3.2.1).
        let count = (size - 16) / 16;

        let mut fcr_free_chunk = Vec::new();
        for _i in 0..count {
            fcr_free_chunk.push(FileChunkReference64::parse(reader)?);
        }

        Ok(Self {
            size,
            crc,
            fcr_next_chunk,
            fcr_free_chunk,
        })
    }
}
