use crate::one::property::{simple, PropertyType};
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Author(String);

impl Author {
    pub(crate) fn name(&self) -> &str {
        &self.0
    }

    pub(crate) fn parse(object: &Object) -> Option<Author> {
        simple::parse_string(PropertyType::Author, object).map(Author)
    }
}
