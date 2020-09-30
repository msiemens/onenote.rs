use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) data: Vec<u8>,
    pub(crate) extension: Option<String>,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert!(
        object.id() == PropertySetId::PictureContainer.as_jcid()
            || object.id() == PropertySetId::XpsContainer.as_jcid()
    );

    let data = object.file_data().map(|v| v.to_vec()).unwrap_or_default();
    let extension = simple::parse_string(PropertyType::PictureFileExtension, object);

    Data { data, extension }
}
