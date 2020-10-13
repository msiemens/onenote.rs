use crate::errors::Result;
use crate::one::property::{simple, PropertyType};
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Author(String);

impl Author {
    pub(crate) fn into_value(self) -> String {
        self.0
    }

    pub(crate) fn name(&self) -> &str {
        &self.0
    }

    pub(crate) fn parse(object: &Object) -> Result<Option<Author>> {
        Ok(simple::parse_string(PropertyType::Author, object)?.map(Author))
    }
}
