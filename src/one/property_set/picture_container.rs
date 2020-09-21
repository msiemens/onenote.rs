use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;

#[derive(Debug)]
pub(crate) struct Data(Vec<u8>);

impl Data {
    pub(crate) fn data(&self) -> &[u8] {
        &self.0
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::PictureContainer.as_jcid());

    Data(object.file_data().map(|v| v.to_vec()).unwrap_or_default())
}
