use crate::Reader;
use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::fsshttpb::data::object_types::ObjectType;
use crate::fsshttpb::data::stream_object::ObjectHeader;
use crate::fsshttpb::data_element::DataElementPackage;
use crate::shared::guid::Guid;

/// A OneNote file packaged in FSSHTTPB format.
///
/// See [\[MS-ONESTORE\] 2.8.1]
///
/// [\[MS-ONESTORE\] 2.8.1]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/a2f046ea-109a-49c4-912d-dc2888cf0565
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct OneStorePackaging {
    pub(crate) file_type: Guid,
    pub(crate) file: Guid,
    pub(crate) legacy_file_version: Guid,
    pub(crate) file_format: Guid,
    pub(crate) storage_index: ExGuid,
    pub(crate) cell_schema: Guid,
    pub(crate) data_element_package: DataElementPackage,
}

impl OneStorePackaging {
    pub(crate) fn parse(reader: Reader) -> Result<OneStorePackaging> {
        let start = reader.remaining();

        let file_type = Guid::parse(reader)?;
        let file = Guid::parse(reader)?;
        let legacy_file_version = Guid::parse(reader)?;
        let file_format = Guid::parse(reader)?;

        let revision_store_format = guid!("109ADD3F-911B-49F5-A5D0-1791EDC8AED8");
        let package_store_format = guid!("638DE92F-A6D4-4BC1-9A36-B3FC2511A5B7");

        if file_format == package_store_format {
            return Self::parse_legacy(reader, file_type, file, legacy_file_version, file_format);
        }

        if file_format != revision_store_format {
            return Err(ErrorKind::MalformedOneStoreData(
                format!("unknown file format: {file_format}").into(),
            )
            .into());
        }

        if !legacy_file_version.is_nil() {
            return Self::parse_legacy(reader, file_type, file, legacy_file_version, file_format);
        }

        Self::parse_packaging(
            reader,
            file_type,
            file,
            legacy_file_version,
            file_format,
            start as u64,
        )
    }

    fn parse_packaging(
        reader: Reader,
        _file_type: Guid,
        _file: Guid,
        _legacy_file_version: Guid,
        _file_format: Guid,
        start: u64,
    ) -> Result<OneStorePackaging> {
        let last_writer_version = reader.get_u32()?;
        if last_writer_version != 0x0000002A && last_writer_version != 0x0000001B {
            return Err(
                ErrorKind::MalformedFssHttpBData("unknown last writer version".into()).into(),
            );
        }

        let oldest_writer_version = reader.get_u32()?;
        if oldest_writer_version != 0x0000002A && oldest_writer_version != 0x0000001B {
            return Err(
                ErrorKind::MalformedFssHttpBData("unknown oldest writer version".into()).into(),
            );
        }

        let newest_writer_version = reader.get_u32()?;
        if newest_writer_version != 0x0000002A && newest_writer_version != 0x0000001B {
            return Err(
                ErrorKind::MalformedFssHttpBData("unknown newest writer version".into()).into(),
            );
        }

        let oldest_reader_version = reader.get_u32()?;
        if oldest_reader_version != 0x0000002A && oldest_reader_version != 0x0000001B {
            return Err(
                ErrorKind::MalformedFssHttpBData("unknown oldest reader version".into()).into(),
            );
        }

        let _legacy_free_chunk_list = reader.get_u64()?;
        let _legacy_transaction_log = reader.get_u64()?;
        let _transaction_count = reader.get_u32()?;
        let _legacy_expected_file_length = reader.get_u32()?;
        let _placeholer = reader.get_u64()?;
        let _legacy_file_node_list_root = reader.get_u64()?;
        let _legacy_free_space_in_chunk_list = reader.get_u32()?;
        let _needs_defrag = reader.get_u8()?;
        let _repaired_file = reader.get_u8()?;
        let _needs_garbage_collect = reader.get_u8()?;
        let _has_no_embedded_file_objects = reader.get_u8()?;
        let _guid_ancestor = Guid::parse(reader)?;
        let _crc_name = reader.get_u32()?;
        let _hashed_chunk_list = Self::parse_fcr64x32(reader)?;
        let transaction_log = Self::parse_fcr64x32(reader)?;

        let read = start - reader.remaining() as u64;
        let offset = transaction_log.end()?.checked_sub(read).ok_or_else(|| {
            ErrorKind::MalformedOneStoreData("transaction log offset underflow".into())
        })?;
        if offset > usize::MAX as u64 {
            return Err(ErrorKind::MalformedOneStoreData(
                "transaction log offset exceeds addressable space".into(),
            )
            .into());
        }
        reader.advance(offset as usize)?;

        let file_type = Guid::parse(reader)?;
        let file = Guid::parse(reader)?;
        let legacy_file_version = Guid::parse(reader)?;
        let file_format = Guid::parse(reader)?;

        Self::parse_legacy(reader, file_type, file, legacy_file_version, file_format)
    }

    fn parse_legacy(
        reader: Reader,
        file_type: Guid,
        file: Guid,
        legacy_file_version: Guid,
        file_format: Guid,
    ) -> Result<OneStorePackaging> {
        if reader.get_u32()? != 0 {
            return Err(ErrorKind::MalformedFssHttpBData("invalid padding data".into()).into());
        }

        ObjectHeader::try_parse_32(reader, ObjectType::OneNotePackaging)?;

        let storage_index = ExGuid::parse(reader)?;
        let cell_schema = Guid::parse(reader)?;

        let data_element_package = DataElementPackage::parse(reader)?;

        ObjectHeader::try_parse_end_16(reader, ObjectType::OneNotePackaging)?;

        Ok(OneStorePackaging {
            file_type,
            file,
            legacy_file_version,
            file_format,
            storage_index,
            cell_schema,
            data_element_package,
        })
    }

    fn parse_fcr64x32(reader: &mut crate::reader::Reader) -> Result<FileChunkReference64x32> {
        let stp = reader.get_u64()?;
        let cb = reader.get_u32()?;

        Ok(FileChunkReference64x32 { stp, cb })
    }
}

#[derive(Debug)]
struct FileChunkReference64x32 {
    stp: u64,
    cb: u32,
}

impl FileChunkReference64x32 {
    fn end(&self) -> Result<u64> {
        self.stp.checked_add(self.cb as u64).ok_or_else(|| {
            ErrorKind::MalformedOneStoreData("file chunk reference overflow".into()).into()
        })
    }
}
