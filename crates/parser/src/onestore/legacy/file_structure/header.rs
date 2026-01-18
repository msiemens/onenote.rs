use super::super::common::{FileChunkReference32, FileChunkReference64x32};
use crate::Reader;
use crate::errors::{ErrorKind, Result};
use crate::onestore::legacy::parse::Parse;
use crate::shared::guid::Guid;
use parser_macros::Parse;

/// A OneNote file header in the standard OneNote 2016 format.
///
/// See [\[MS-ONESTORE\] 2.3.1]
///
/// [\[MS-ONESTORE\] 2.3.1]: https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/2b394c6b-8788-441f-b631-da1583d772fd
///
#[derive(Debug, Parse)]
#[allow(dead_code)]
#[validate({
    // .one file
    file_type == guid!("{7B5C52E4-D88C-4DA7-AEB1-5378D02996D3}")
    // .onetoc2 file
    || file_type == guid!("{43FF2FA1-EFD9-4C76-9EE2-10EA5722765F}")
})]
pub(crate) struct OneStoreHeader {
    pub(crate) file_type: Guid,
    _guid_file: Guid,
    pub(crate) legacy_file_version: Guid,
    pub(crate) file_format: OneStoreFormatGuid,
    pub(crate) ffv_last_code_that_wrote_to_this_file: u32,
    pub(crate) ffv_oldest_code_that_has_written_to_this_file: u32,
    pub(crate) ffv_newest_code_that_has_written_to_this_file: u32,
    pub(crate) ffv_oldest_code_that_may_read_this_file: u32,
    pub(crate) fcr_legacy_free_chunk_list: FileChunkReference32,
    pub(crate) fcr_legacy_transaction_log: FileChunkReference32,
    pub(crate) c_transactions_in_log: u32,
    pub(crate) cb_legacy_expected_file_length: u32,
    #[assert_offset(104)]
    pub(crate) rgb_placeholder: u64,
    pub(crate) fcr_legacy_file_node_list_root: FileChunkReference32,
    pub(crate) cb_legacy_free_space_in_free_chunk_list: u32,
    pub(crate) f_needs_defrag: u8,
    pub(crate) f_repaired_file: u8,
    pub(crate) f_needs_garbage_collect: u8,
    pub(crate) f_has_no_embedded_file_objects: u8,
    pub(crate) guid_ancestor: Guid,
    #[assert_offset(144)]
    pub(crate) crc_name: u32,
    pub(crate) fcr_hashed_chunk_list: FileChunkReference64x32,
    pub(crate) fcr_transaction_log: FileChunkReference64x32,
    pub(crate) fcr_file_node_list_root: FileChunkReference64x32,
    pub(crate) fcr_free_chunk_list: FileChunkReference64x32,
    pub(crate) cb_expected_file_length: u64,
    pub(crate) cb_free_space_in_free_chunk_list: u64,
    pub(crate) guid_file_version: Guid,
    #[assert_offset(228)]
    pub(crate) n_file_version_generation: u64,
    pub(crate) guid_deny_read_file_version: Guid,
    pub(crate) grf_debug_log_flags: u32,
    pub(crate) fcr_debug_log: FileChunkReference64x32,
    pub(crate) fcr_alloc_verification_free_chunk_list: FileChunkReference64x32,
    pub(crate) bn_created: u32,
    pub(crate) bn_last_wrote_to_this_file: u32,
    pub(crate) bn_oldest_written: u32,
    pub(crate) bn_newest_written: u32,
    #[assert_offset(296)]
    pub(crate) rgb_reserved: RgbReserved,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct OneStoreFormatGuid {
    value: Guid,
}

impl Parse for OneStoreFormatGuid {
    fn parse(reader: Reader) -> Result<Self> {
        let file_format = Guid::parse(reader)?;
        if file_format != Guid::from_str("109ADD3F-911B-49F5-A5D0-1791EDC8AED8")? {
            // Matches the file format specified in MS-ONESTORE section 2.3
            return Err(ErrorKind::NotLocalOneStore(
                "This parser only supports OneNote^(r) 2016-style Notebooks.".into(),
            )
            .into());
        }

        Ok(Self { value: file_format })
    }
}

#[derive(Debug)]
pub(crate) struct RgbReserved {}

impl Parse for RgbReserved {
    fn parse(reader: Reader) -> Result<Self> {
        // Skip
        reader.advance(728)?;

        Ok(Self {})
    }
}
