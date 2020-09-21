use crate::one::property::author::Author;

use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Data(Author);

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::AuthorContainer.as_jcid());

    let author = Author::parse(object).expect("author container has not author field");

    Data(author)
}
