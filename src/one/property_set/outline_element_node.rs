use crate::one::property::object_reference::ObjectReference;
use crate::one::property::time::Time;
use crate::one::property::{simple, PropertyType};
use crate::one::property_set::PropertySetId;
use crate::onestore::object::Object;
use crate::types::exguid::ExGuid;

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) created_at: Time,
    pub(crate) last_modified: Time,
    pub(crate) children: Vec<ExGuid>,
    pub(crate) child_level: u8,
    pub(crate) contents: Vec<ExGuid>,
    pub(crate) list_contents: Vec<ExGuid>,
    pub(crate) list_spacing: Option<f32>,
    pub(crate) author_original: ExGuid,
    pub(crate) author_most_recent: ExGuid,
    pub(crate) rtl: bool,
    pub(crate) is_deletable: bool,
    pub(crate) is_selectable: bool,
    pub(crate) is_title_text: bool,
}

pub(crate) fn parse(object: &Object) -> Data {
    assert_eq!(object.id(), PropertySetId::OutlineElementNode.as_jcid());

    let created_at = Time::parse(PropertyType::CreationTimeStamp, object)
        .expect("outline element has no creation timestamp");
    let last_modified = Time::parse(PropertyType::LastModifiedTime, object)
        .expect("outline element has no last modified time");
    let children =
        ObjectReference::parse_vec(PropertyType::ElementChildNodes, object).unwrap_or_default();
    let child_level = simple::parse_u8(PropertyType::OutlineElementChildLevel, object)
        .expect("outline has no child element level");
    let contents =
        ObjectReference::parse_vec(PropertyType::ContentChildNodes, object).unwrap_or_default();
    let list_contents =
        ObjectReference::parse_vec(PropertyType::ListNodes, object).unwrap_or_default();
    let list_spacing = simple::parse_f32(PropertyType::ListSpacingMu, object);
    let author_original = ObjectReference::parse(PropertyType::AuthorOriginal, object)
        .expect("outline element has no original author");
    let author_most_recent = ObjectReference::parse(PropertyType::AuthorMostRecent, object)
        .expect("outline element has no most recent author");
    let rtl = simple::parse_bool(PropertyType::OutlineElementRTL, object).unwrap_or_default();
    let is_deletable = simple::parse_bool(PropertyType::Deletable, object).unwrap_or_default();
    let is_selectable = simple::parse_bool(PropertyType::CannotBeSelected, object)
        .map(|value| !value)
        .unwrap_or_default();
    let is_title_text = simple::parse_bool(PropertyType::IsTitleText, object).unwrap_or_default();

    Data {
        created_at,
        last_modified,
        children,
        child_level,
        contents,
        list_contents,
        list_spacing,
        author_original,
        author_most_recent,
        rtl,
        is_deletable,
        is_selectable,
        is_title_text,
    }
}
