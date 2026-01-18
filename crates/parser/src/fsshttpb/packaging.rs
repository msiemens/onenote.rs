use crate::Reader;
use crate::errors::{ErrorKind, Result};
use crate::fsshttpb::data::exguid::ExGuid;
use crate::fsshttpb::data::object_types::ObjectType;
use crate::fsshttpb::data::stream_object::ObjectHeader;
use crate::fsshttpb::data_element::DataElementPackage;
use crate::onestore::legacy::file_structure::OneStoreHeader;
use crate::onestore::legacy::parse::Parse;
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
        let file_type = Guid::parse(reader)?;
        let file = Guid::parse(reader)?;
        let legacy_file_version = Guid::parse(reader)?;
        let file_format = Guid::parse(reader)?;
        let package_store_format = guid!("638DE92F-A6D4-4BC1-9A36-B3FC2511A5B7");

        if file_format != package_store_format {
            return Err(ErrorKind::MalformedOneStoreData(
                format!("unknown file format: {file_format}").into(),
            )
            .into());
        }

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
}

/// Return the byte offset where an embedded FSSHTTPB packaging structure starts.
///
/// # Implementation notes:
/// - The embedded packaging lives *after* the legacy header/transaction log, so we parse
///   `OneStoreHeader` to compute the end of the transaction log and use that as the candidate
///   offset.
/// - We only accept files that look like revision-store headers with a nil legacy version
///   (modern OneDrive downloads), then validate that the GUIDs at the computed offset match
///   a package store file before returning the offset.
/// - Using the transaction-log end as the packaging start appears undocumented in the spec;
///   we rely on observed file layouts.
pub(crate) fn embedded_packaging_offset(data: &[u8]) -> Option<usize> {
    let mut reader = crate::reader::Reader::new(data);
    let start = reader.remaining();

    let (header, end) = parse_header_log_end(&mut reader, start as u64).ok()?;
    if !header.legacy_file_version.is_nil() {
        return None;
    }
    let offset = usize::try_from(end).ok()?;

    if offset + 16 * 4 > data.len() {
        return None;
    }

    let mut reader = crate::reader::Reader::new(data)
        .with_updated_bounds(offset, data.len())
        .ok()?;

    let _file_type = Guid::parse(&mut reader).ok()?;
    let _file = Guid::parse(&mut reader).ok()?;
    let _legacy_file_version = Guid::parse(&mut reader).ok()?;
    let file_format = Guid::parse(&mut reader).ok()?;

    let package_store_format = guid!("638DE92F-A6D4-4BC1-9A36-B3FC2511A5B7");
    if file_format == package_store_format {
        Some(offset)
    } else {
        None
    }
}

/// Parses the legacy header and returns the end offset of the transaction log.
///
/// Implementation notes:
/// - The legacy header format is specified in [MS-ONESTORE] 2.3.1, but the use of the
///   transaction-log end as an embedded packaging locator appears undocumented.
fn parse_header_log_end(reader: Reader, start: u64) -> Result<(OneStoreHeader, u64)> {
    let header = OneStoreHeader::parse(reader)?;
    let end = header
        .fcr_transaction_log
        .stp
        .checked_add(header.fcr_transaction_log.cb as u64)
        .ok_or_else(|| {
            ErrorKind::MalformedOneStoreData("transaction log offset overflow".into())
        })?;
    let read = start - reader.remaining() as u64;
    if end < read {
        return Err(
            ErrorKind::MalformedOneStoreData("transaction log offset underflow".into()).into(),
        );
    }

    Ok((header, end))
}
