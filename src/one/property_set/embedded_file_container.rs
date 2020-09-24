use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;


#[derive(Debug)]
pub(crate) struct Data(pub(crate) Vec<u8>);

impl Data {
    pub(crate) fn into_value(self) -> Vec<u8> {
        self.0
    }
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::EmbeddedFileContainer.as_jcid());

    Data(
        object
            .file_data()
            .expect("embedded file container has no data")
            .to_vec(),
    )
}
