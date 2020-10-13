use crate::errors::{ErrorKind, Result};
use crate::one::property::author::Author;
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Data(Author);

pub(crate) fn parse(object: &Object) -> Result<Data> {
    if object.id() != PropertySetId::AuthorContainer.as_jcid() {
        return Err(ErrorKind::MalformedOneNoteFileData(
            format!("unexpected object type: 0x{:X}", object.id().0).into(),
        )
        .into());
    }

    let author = Author::parse(object)?.ok_or_else(|| {
        ErrorKind::MalformedOneNoteFileData("author container has not author field".into())
    })?;

    Ok(Data(author))
}
