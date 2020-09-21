use crate::one::property::PropertyType;
use crate::onestore::object::Object;
use crate::types::guid::Guid;
use crate::utils::Utf16ToString;
use encoding_rs::mem::decode_latin1;

pub(crate) fn parse_bool(prop_type: PropertyType, object: &Object) -> Option<bool> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_bool().expect("bool value is not a bool"))
}

pub(crate) fn parse_u8(prop_type: PropertyType, object: &Object) -> Option<u8> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_u8().expect("u8 value is not a u8"))
}

pub(crate) fn parse_u16(prop_type: PropertyType, object: &Object) -> Option<u16> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_u16().expect("u16 value is not a u16"))
}

pub(crate) fn parse_u32(prop_type: PropertyType, object: &Object) -> Option<u32> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_u32().expect("u32 value is not a u32"))
}

pub(crate) fn parse_u64(prop_type: PropertyType, object: &Object) -> Option<u64> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_u64().expect("u64 value is not a u64"))
}

pub(crate) fn parse_f32(prop_type: PropertyType, object: &Object) -> Option<f32> {
    object.props().get(prop_type).map(|value| {
        f32::from_le_bytes(
            value
                .to_u32()
                .expect("float value is not a u32")
                .to_le_bytes(),
        )
    })
}

pub(crate) fn parse_vec(prop_type: PropertyType, object: &Object) -> Option<Vec<u8>> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_vec().expect("vec value is not a vec").to_vec())
}

pub(crate) fn parse_vec_u16(prop_type: PropertyType, object: &Object) -> Option<Vec<u16>> {
    object.props().get(prop_type).map(|value| {
        value
            .to_vec()
            .expect("vec u16 value is not a vec")
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect()
    })
}

pub(crate) fn parse_vec_u32(prop_type: PropertyType, object: &Object) -> Option<Vec<u32>> {
    object.props().get(prop_type).map(|value| {
        value
            .to_vec()
            .expect("vec u32 value is not a vec")
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
            .collect()
    })
}

pub(crate) fn parse_ascii(prop_type: PropertyType, object: &Object) -> Option<String> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_vec().expect("ascii value is not a vec"))
        .map(|value| decode_latin1(value).to_string())
}

pub(crate) fn parse_string(prop_type: PropertyType, object: &Object) -> Option<String> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_vec().expect("string value is not a vec"))
        .map(|value| value.utf16_to_string().expect("invalid string"))
}

pub(crate) fn parse_guid(prop_type: PropertyType, object: &Object) -> Option<Guid> {
    object
        .props()
        .get(prop_type)
        .map(|value| value.to_vec().expect("guid value is not a vec"))
        .map(|ref mut value| Guid::parse(value))
}
