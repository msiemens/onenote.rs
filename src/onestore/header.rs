use crate::fsshttpb::data_element::object_group::{ObjectGroup, ObjectGroupData};
use crate::one::property::PropertyType;
use crate::onestore::types::object_prop_set::ObjectPropSet;
use crate::onestore::types::property::PropertyValue;
use crate::types::guid::Guid;

#[derive(Debug)]
pub(crate) struct StoreHeader {
    file_identity: Guid,
    ancestor_identity: Guid,
    last_code_version_that_wrote_to_it: Option<u32>,
    file_name_crc: u32,
}

impl StoreHeader {
    pub(crate) fn parse(data: &ObjectGroup) -> StoreHeader {
        let (_, object_data) = data
            .declarations
            .iter()
            .zip(data.objects.iter())
            .find(|(decl, _)| decl.partition_id() == 1)
            .expect("object data is missing");

        let object_data = if let ObjectGroupData::Object { data, .. } = object_data {
            data
        } else {
            panic!("object group data it not an object")
        };

        let prop_set = ObjectPropSet::parse(&mut object_data.as_slice());

        let file_identity = prop_set
            .get(PropertyType::FileIdentityGuid)
            .map(|value| StoreHeader::parse_guid(value))
            .expect("FileIdentityGuid prop missing");

        let ancestor_identity = prop_set
            .get(PropertyType::FileAncestorIdentityGuid)
            .map(|value| StoreHeader::parse_guid(value))
            .expect("FileAncestorIdentityGuid prop missing");

        let last_code_version_that_wrote_to_it = prop_set
            .get(PropertyType::FileLastCodeVersionThatWroteToIt)
            .map(|value| StoreHeader::parse_u32(value));

        let file_name_crc = prop_set
            .get(PropertyType::FileNameCRC)
            .map(|value| StoreHeader::parse_u32(value))
            .expect("FileNameCRC prop missing");

        StoreHeader {
            file_identity,
            ancestor_identity,
            last_code_version_that_wrote_to_it,
            file_name_crc,
        }
    }

    fn parse_guid(value: &PropertyValue) -> Guid {
        if let PropertyValue::Vec(data) = &value {
            Guid::parse(&mut data.as_slice())
        } else {
            panic!("property is not a vec")
        }
    }

    fn parse_u32(value: &PropertyValue) -> u32 {
        if let PropertyValue::U32(v) = value {
            *v
        } else {
            panic!("property is not a vec")
        }
    }
}
