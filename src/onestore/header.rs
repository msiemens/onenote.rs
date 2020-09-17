use crate::fsshttpb::data_element::object_group::{
    ObjectGroup, ObjectGroupData, ObjectGroupDeclaration,
};
use crate::onestore::types::object_prop_set::ObjectPropSet;
use crate::onestore::types::property::PropertyValue;
use crate::types::guid::Guid;

#[derive(Debug)]
pub(crate) struct StoreHeader {
    file_identity: Guid,
    ancestor_identity: Guid,
    last_code_version_that_wrote_to_it: u32,
    file_name_crc: u32,
}

impl StoreHeader {
    pub(crate) fn parse(data: &ObjectGroup) -> StoreHeader {
        let declaration = data
            .declarations
            .first()
            .expect("no header object declaration");
        if let ObjectGroupDeclaration::Object { partition_id, .. } = declaration {
            assert_eq!(*partition_id, 1);
        } else {
            panic!("object group declaration it not an object")
        };

        let object_data = data.objects.first().expect("no header object data");
        let object_data = if let ObjectGroupData::Object { data, .. } = object_data {
            data
        } else {
            panic!("object group data it not an object")
        };

        let prop_set = ObjectPropSet::parse(&mut object_data.as_slice());

        let file_identity = StoreHeader::parse_guid(
            prop_set
                .get(0x1C001D94)
                .expect("FileIdentityGuid prop missing"),
        );

        let ancestor_identity = StoreHeader::parse_guid(
            prop_set
                .get(0x1C001D95)
                .expect("FileAncestorIdentityGuid prop missing"),
        );

        let last_code_version_that_wrote_to_it = StoreHeader::parse_u32(
            prop_set
                .get(0x14001D99)
                .expect("FileLastCodeVersionThatWroteToIt prop missing"),
        );

        let file_name_crc = StoreHeader::parse_u32(
            prop_set
                .get(0x14001D99)
                .expect("FileLastCodeVersionThatWroteToIt prop missing"),
        );

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
