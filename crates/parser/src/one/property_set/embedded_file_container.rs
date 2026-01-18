use crate::errors::{ErrorKind, Result};
use crate::one::property_set::{PropertySetId, assert_property_set};
use crate::onestore::Object;
use crate::onestore::shared::file_blob::FileBlob;

/// An embedded file data container.
///
/// See [\[MS-ONE\] 2.2.59].
///
/// [\[MS-ONE\] 2.2.59]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/e2a23dc5-75a5-407f-b5ff-d3412379fa7b
#[derive(Debug)]
pub(crate) struct Data(pub(crate) FileBlob);

impl Data {
    pub(crate) fn into_value(self) -> FileBlob {
        self.0
    }
}

pub(crate) fn parse(object: &Object) -> Result<Data> {
    assert_property_set(object, PropertySetId::EmbeddedFileContainer)?;

    let data = object.file_data.clone().ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("embedded file container has no data".into())
    })?;

    Ok(Data(data))
}
